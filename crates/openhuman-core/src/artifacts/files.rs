//! Where an artifact's bytes live (#5505).
//!
//! An artifact's metadata stays in the hidden, per-account
//! `<workspace_dir>/artifacts/<id>/`, but the file itself is written to a
//! visible folder the user can open (`~/OpenHuman/projects/Files` by default,
//! [`crate::config::default_files_dir`]). Files there are named for people —
//! `<sanitized-title>.<ext>`, then `<sanitized-title> (2).<ext>` on collision —
//! and each record carries the absolute `file` it owns plus the `file_root` it
//! was placed in.
//!
//! The folder is shared by every account on the OS user, so a name is claimed
//! with `create_new` (an atomic "fail if it exists"): two accounts, or two
//! concurrent generations, can never be handed the same path.
//!
//! # The escape guard
//!
//! `meta.json` is data, so a record's `file` is checked before anything reads
//! or copies it. Its `file_root` counts only when it is one of the folders the
//! core vouches for ([`FileRoots`], built by the caller or from config — never
//! taken from the record), and the file must be absolute, free of `..`,
//! strictly inside that root (after resolving symlinks when it exists), and never a
//! path [`SecurityPolicy::is_always_forbidden`] rejects. That last check is the
//! same floor the agent's own file tools keep when the autonomy policy is off,
//! so a hand-edited record cannot reach further than the agent already could.
//! (`artifacts/` is deliberately not a workspace-internal dir: the agent reads
//! its own large tool outputs back from `artifacts/tool-results/`.)

use std::path::{Component, Path, PathBuf};

use tokio::io::AsyncWriteExt;

use super::store::{artifacts_root, get_artifact};
use super::types::{ArtifactMeta, ArtifactStatus};
use crate::security::SecurityPolicy;

/// The files folders the core vouches for: where new files go, plus every
/// folder an existing record may legitimately point into. A record's
/// `file_root` is honoured only when it matches one of these, so a
/// hand-edited `meta.json` cannot widen what Download and
/// `read_artifact_bytes` will serve.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileRoots {
    current: PathBuf,
    trusted: Vec<PathBuf>,
}

impl FileRoots {
    /// New files go to `current`, which is also the only trusted folder.
    pub fn new(current: impl Into<PathBuf>) -> Self {
        let current = current.into();
        Self {
            trusted: vec![current.clone()],
            current,
        }
    }

    /// Also trust `folders` for existing records.
    pub fn with_trusted(mut self, folders: impl IntoIterator<Item = PathBuf>) -> Self {
        for folder in folders {
            if !self.trusted.contains(&folder) {
                self.trusted.push(folder);
            }
        }
        self
    }

    /// The folders for this host config: new files go to the configured
    /// folder, and records made in the default folder or any folder used
    /// before (`files_dir_history`) keep resolving.
    pub fn from_config(config: &crate::config::Config) -> Self {
        Self::new(config.files_dir()).with_trusted(
            std::iter::once(crate::config::default_files_dir())
                .chain(config.files_dir_history.iter().cloned()),
        )
    }

    /// Where new files are written.
    pub fn current(&self) -> &Path {
        &self.current
    }

    /// Every vouched-for folder: [`Self::current`] and the trusted ones.
    pub fn trusted(&self) -> &[PathBuf] {
        &self.trusted
    }

    /// Whether `root` is one of the vouched-for folders, compared canonically
    /// where the paths exist.
    fn trusts(&self, root: &Path) -> bool {
        let canon = |p: &Path| p.canonicalize().unwrap_or_else(|_| p.to_path_buf());
        let root = canon(root);
        self.trusted.iter().any(|t| canon(t) == root)
    }

    /// Whether an existing path is beneath any folder this host trusts for
    /// artifact resolution, including prior Files-folder locations.
    pub(crate) fn contains_path(&self, path: &Path) -> bool {
        let canonical_path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
        self.trusted.iter().any(|root| {
            let canonical_root = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
            path.starts_with(root) || canonical_path.starts_with(&canonical_root)
        })
    }
}

impl From<PathBuf> for FileRoots {
    fn from(current: PathBuf) -> Self {
        Self::new(current)
    }
}

impl From<&PathBuf> for FileRoots {
    fn from(current: &PathBuf) -> Self {
        Self::new(current.clone())
    }
}

impl From<&Path> for FileRoots {
    fn from(current: &Path) -> Self {
        Self::new(current.to_path_buf())
    }
}

impl From<&FileRoots> for FileRoots {
    fn from(roots: &FileRoots) -> Self {
        roots.clone()
    }
}

/// Replace a visible artifact projection only inside a folder vouched for by
/// the host. Artifact records and version data remain in their storage backend;
/// this is the explicit publication boundary for the operator-visible Files
/// folders.
pub(crate) async fn write_visible_projection(
    ctx: &crate::core::runtime::DispatchCtx,
    roots: &FileRoots,
    destination: &Path,
    bytes: &[u8],
) -> Result<u64, String> {
    let destination = resolve_visible_projection_destination(roots, destination).await?;
    let (staged, mut output) = StagedProjection::create(destination)
        .map_err(|error| format!("opening visible artifact failed: {error}"))?;
    output
        .write_all(bytes)
        .await
        .map_err(|error| format!("writing visible artifact failed: {error}"))?;
    staged.publish(ctx, output).await?;
    Ok(bytes.len() as u64)
}

/// Copy a file-backed artifact into its validated Files projection without
/// buffering the entire artifact in memory.
pub(crate) async fn copy_visible_projection(
    ctx: &crate::core::runtime::DispatchCtx,
    roots: &FileRoots,
    destination: &Path,
    source: &Path,
) -> Result<u64, String> {
    let destination = resolve_visible_projection_destination(roots, destination).await?;
    let mut input = tokio::fs::File::open(source)
        .await
        .map_err(|error| format!("opening artifact source failed: {error}"))?;
    let (staged, mut output) = StagedProjection::create(destination)
        .map_err(|error| format!("opening visible artifact failed: {error}"))?;
    let copied = tokio::io::copy(&mut input, &mut output)
        .await
        .map_err(|error| format!("copying visible artifact failed: {error}"))?;
    staged.publish(ctx, output).await?;
    Ok(copied)
}

struct ProjectionDestination {
    parent: cap_std::fs::Dir,
    name: std::ffi::OsString,
}

/// Only a fresh inode receives bytes. Rename replaces a directory entry, so a
/// hardlink or a leaf symlink swapped after validation cannot redirect writes.
/// Relative operations remain bound to the admitted parent directory handle.
struct StagedProjection {
    destination: ProjectionDestination,
    temporary: String,
}

impl StagedProjection {
    fn create(destination: ProjectionDestination) -> std::io::Result<(Self, tokio::fs::File)> {
        let temporary = format!(".openhuman-{}.tmp", uuid::Uuid::new_v4());
        let file = destination
            .parent
            .open_with(&temporary, &artifact_open_options())?;
        Ok((
            Self {
                destination,
                temporary,
            },
            tokio::fs::File::from_std(file.into_std()),
        ))
    }

    async fn publish(
        self,
        ctx: &crate::core::runtime::DispatchCtx,
        mut output: tokio::fs::File,
    ) -> Result<(), String> {
        output
            .flush()
            .await
            .map_err(|error| format!("flushing visible artifact failed: {error}"))?;
        output
            .sync_all()
            .await
            .map_err(|error| format!("syncing visible artifact failed: {error}"))?;
        drop(output.into_std().await);
        ctx.spawn_blocking(move || {
            self.destination.parent.rename(
                &self.temporary,
                &self.destination.parent,
                &self.destination.name,
            )
        })
        .await
        .map_err(|error| format!("publishing visible artifact failed: {error}"))?
        .map_err(|error| format!("publishing visible artifact failed: {error}"))
    }
}

impl Drop for StagedProjection {
    fn drop(&mut self) {
        // After publication this name no longer exists. On failure discard only
        // the fresh staging entry through the same anchored directory handle.
        let _ = self.destination.parent.remove_file(&self.temporary);
    }
}

async fn resolve_visible_projection_destination(
    roots: &FileRoots,
    destination: &Path,
) -> Result<ProjectionDestination, String> {
    if !destination.is_absolute() || has_parent_component(destination) {
        return Err("visible artifact must be inside a trusted Files folder".into());
    }
    let file_name = destination
        .file_name()
        .filter(|name| !name.is_empty())
        .ok_or_else(|| "visible artifact must be inside a trusted Files folder".to_string())?;
    let parent = destination
        .parent()
        .ok_or_else(|| "visible artifact must be inside a trusted Files folder".to_string())?;

    let mut resolved_destination = None;
    for root in roots.trusted() {
        if !root.is_absolute() || has_parent_component(root) || !destination.starts_with(root) {
            continue;
        }
        let canonical_root = root
            .canonicalize()
            .map_err(|_| "trusted Files folder could not be resolved".to_string())?;
        let canonical_parent = parent
            .canonicalize()
            .map_err(|_| "visible artifact parent could not be resolved".to_string())?;
        if !canonical_parent.starts_with(&canonical_root) {
            continue;
        }
        let target = canonical_parent.join(file_name);
        if crate::security::SecurityPolicy::is_always_forbidden(&target) {
            return Err("visible artifact is in a protected location".into());
        }
        match tokio::fs::symlink_metadata(&target).await {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err("visible artifact may not replace a symlink".into());
            }
            Ok(_) => {
                let resolved = target
                    .canonicalize()
                    .map_err(|_| "visible artifact could not be resolved".to_string())?;
                if !resolved.starts_with(&canonical_root)
                    || crate::security::SecurityPolicy::is_always_forbidden(&resolved)
                {
                    return Err("visible artifact resolves outside its trusted Files folder".into());
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(format!("visible artifact could not be checked: {error}")),
        }
        let parent = open_projection_parent(&canonical_parent)
            .map_err(|error| format!("opening visible artifact parent failed: {error}"))?;
        resolved_destination = Some(ProjectionDestination {
            parent,
            name: file_name.to_owned(),
        });
        break;
    }
    resolved_destination
        .ok_or_else(|| "visible artifact must be inside a trusted Files folder".to_string())
}

/// Walk the already-canonical path from its filesystem root, refusing links at
/// every component. Opening the whole path ambiently after validation would
/// let a root/ancestor swap redirect the handle before capability confinement.
fn open_projection_parent(path: &Path) -> std::io::Result<cap_std::fs::Dir> {
    use cap_fs_ext::DirExt;
    let mut anchor = PathBuf::new();
    let mut names = Vec::new();
    for component in path.components() {
        match component {
            Component::Prefix(_) | Component::RootDir if names.is_empty() => {
                anchor.push(component.as_os_str());
            }
            Component::Normal(name) => names.push(name),
            _ => {
                return Err(std::io::Error::other(
                    "invalid canonical artifact directory",
                ))
            }
        }
    }
    if !anchor.is_absolute() {
        return Err(std::io::Error::other("artifact directory must be absolute"));
    }
    let mut directory = cap_std::fs::Dir::open_ambient_dir(anchor, cap_std::ambient_authority())?;
    for name in names {
        directory = directory.open_dir_nofollow(name)?;
    }
    Ok(directory)
}

/// The shared exclusive-creation policy for reservations and staged publication.
/// Neither path truncates an existing inode or follows a destination leaf link.
fn artifact_open_options() -> cap_std::fs::OpenOptions {
    let mut options = cap_std::fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use cap_std::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    options
}

pub(super) async fn open_artifact_file(path: &Path) -> std::io::Result<tokio::fs::File> {
    let parent = path
        .parent()
        .ok_or_else(|| std::io::Error::other("artifact parent is missing"))?;
    let name = path
        .file_name()
        .ok_or_else(|| std::io::Error::other("artifact name is missing"))?;
    let directory = open_projection_parent(&parent.canonicalize()?)?;
    directory
        .open_with(name, &artifact_open_options())
        .map(|file| tokio::fs::File::from_std(file.into_std()))
}

/// Collision suffixes tried before falling back to an id-tagged name.
const MAX_COLLISION_SUFFIX: u32 = 1000;

/// Claim a fresh, human-named file in `files_dir` for `<stem>.<ext>`.
///
/// Creates `files_dir` when missing, then creates an empty placeholder with
/// `create_new` so the name belongs to this caller before any bytes are
/// generated. Returns the placeholder's absolute path.
pub(crate) async fn reserve_file(
    files_dir: &Path,
    stem: &str,
    ext: &str,
    id: &str,
) -> Result<PathBuf, String> {
    tokio::fs::create_dir_all(files_dir)
        .await
        .map_err(|e| format!("[artifacts] failed to create files folder {files_dir:?}: {e}"))?;
    let candidates = (1..=MAX_COLLISION_SUFFIX)
        .map(|n| {
            if n == 1 {
                format!("{stem}.{ext}")
            } else {
                format!("{stem} ({n}).{ext}")
            }
        })
        .chain(std::iter::once(format!("{stem}-{id}.{ext}")));
    for name in candidates {
        let candidate = files_dir.join(&name);
        match open_artifact_file(&candidate).await {
            Ok(_) => {
                log::debug!("[artifacts] reserve_file: id={id} claimed {name:?}");
                return Ok(candidate);
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(e) => {
                return Err(format!(
                    "[artifacts] failed to reserve {candidate:?} for id={id}: {e}"
                ))
            }
        }
    }
    Err(format!(
        "[artifacts] no free file name for id={id} in {files_dir:?}"
    ))
}

/// Resolve the absolute path of a record's bytes, applying the escape guard.
///
/// A record with `file` set resolves to it; a legacy record resolves to
/// `<workspace>/artifacts/<meta.path>`. Does not require the file to exist.
pub(crate) async fn resolve_file(
    workspace_dir: &Path,
    meta: &ArtifactMeta,
    roots: &FileRoots,
) -> Result<PathBuf, String> {
    let Some(file) = meta.file.as_deref() else {
        let root = artifacts_root(workspace_dir).await?;
        let resolved = root.join(&meta.path);
        if has_parent_component(Path::new(&meta.path)) || !resolved.starts_with(&root) {
            return Err(format!(
                "[artifacts] meta.path {:?} escapes artifacts root for id={}",
                meta.path, meta.id
            ));
        }
        return Ok(resolved);
    };
    let file = PathBuf::from(file);
    let root = meta
        .file_root
        .as_deref()
        .map(PathBuf::from)
        .ok_or_else(|| format!("[artifacts] id={} has a file but no file_root", meta.id))?;
    if !roots.trusts(&root) {
        return Err(format!(
            "[artifacts] file for id={} rejected by the escape guard: its folder is not a files folder",
            meta.id
        ));
    }
    check_within(&root, &file).map_err(|reason| {
        format!(
            "[artifacts] file for id={} rejected by the escape guard: {reason}",
            meta.id
        )
    })?;
    Ok(file)
}

fn has_parent_component(path: &Path) -> bool {
    path.components().any(|c| matches!(c, Component::ParentDir))
}

/// The escape guard for a recorded `file` against its `file_root`.
fn check_within(root: &Path, file: &Path) -> Result<(), &'static str> {
    if !root.is_absolute() || !file.is_absolute() {
        return Err("path is not absolute");
    }
    if has_parent_component(root) || has_parent_component(file) {
        return Err("path contains '..'");
    }
    if file == root || !file.starts_with(root) {
        return Err("file is outside its files folder");
    }
    if SecurityPolicy::is_always_forbidden(file) {
        return Err("file is in a protected location");
    }
    // A symlink inside the folder must not lead out of it. Fail closed: when
    // anything is at the path, both sides must resolve, so a dangling or
    // looping symlink (or an unresolvable folder) is refused rather than
    // skipped — a skipped check would let a later read follow whatever the
    // link comes to point at. Only a path with nothing at it passes here; the
    // caller reports that as "file missing".
    match file.symlink_metadata() {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(_) => return Err("file could not be resolved"),
        Ok(_) => {
            let (Ok(canon_file), Ok(canon_root)) = (file.canonicalize(), root.canonicalize())
            else {
                return Err("file could not be resolved");
            };
            if !canon_file.starts_with(&canon_root)
                || SecurityPolicy::is_always_forbidden(&canon_file)
            {
                return Err("file resolves outside its files folder");
            }
        }
    }
    Ok(())
}

/// Resolve a `Ready` artifact's bytes to an existing file.
///
/// The one sanctioned id → path lookup for anything that reads or exports the
/// file (`read_artifact_bytes`, `ai.get_artifact`, the desktop Download
/// command). Errors when the record is not `Ready`, fails the escape guard, or
/// its file was moved or deleted outside OpenHuman.
pub async fn resolve_ready_file(
    workspace_dir: &Path,
    roots: &FileRoots,
    artifact_id: &str,
) -> Result<PathBuf, String> {
    let meta = get_artifact(workspace_dir, artifact_id).await?;
    resolve_ready_file_meta(workspace_dir, roots, &meta).await
}

/// Resolve an already-loaded `Ready` artifact record to its existing file.
/// Callers that list many records can reuse each row instead of loading it a
/// second time by id.
pub(crate) async fn resolve_ready_file_meta(
    workspace_dir: &Path,
    roots: &FileRoots,
    meta: &ArtifactMeta,
) -> Result<PathBuf, String> {
    if !matches!(&meta.status, ArtifactStatus::Ready) {
        return Err(format!(
            "[artifacts] artifact id={} is not ready (status={:?})",
            meta.id, meta.status
        ));
    }
    let path = resolve_file(workspace_dir, meta, roots).await?;
    if !tokio::fs::try_exists(&path).await.unwrap_or(false) {
        return Err(missing_file_error(&meta.id, &path));
    }
    Ok(path)
}

/// The error for a `Ready` record whose file is gone.
pub(crate) fn missing_file_error(artifact_id: &str, path: &Path) -> String {
    format!(
        "[artifacts] file missing for id={artifact_id}: {} was moved or deleted outside OpenHuman",
        path.display()
    )
}

/// A failed generation must not leave an empty file in the user's files
/// folder: drop the zero-byte placeholder `create_artifact` reserved. A file
/// with bytes (e.g. the previous output of a failed regenerate) is kept.
pub(crate) async fn remove_empty_placeholder(
    workspace_dir: &Path,
    meta: &ArtifactMeta,
    roots: &FileRoots,
) {
    if meta.file.is_none() {
        return;
    }
    let Ok(path) = resolve_file(workspace_dir, meta, roots).await else {
        return;
    };
    if matches!(tokio::fs::metadata(&path).await, Ok(m) if m.is_file() && m.len() == 0) {
        match tokio::fs::remove_file(&path).await {
            Ok(()) => log::debug!("[artifacts] removed empty placeholder for id={}", meta.id),
            Err(e) => log::warn!(
                "[artifacts] could not remove empty placeholder for id={}: {e}",
                meta.id
            ),
        }
    }
}

#[cfg(test)]
#[path = "files_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "files_publication_tests.rs"]
mod publication_tests;
