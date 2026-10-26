use super::*;

/// Where each file version keeps its own copy:
/// `<workspace>/artifacts/versioned/files/<id>/v<N>.<ext>`. The visible file
/// in the files folder shows the newest version.
pub fn version_files_dir(workspace_dir: &Path) -> PathBuf {
    workspace_dir
        .join("artifacts")
        .join(VERSIONED_SUBDIR)
        .join("files")
}

/// Where a version's stored file lives on disk: its storage key under
/// [`version_files_dir`]. A key that is an absolute path (a record written
/// before files were kept behind a store) names that path itself.
pub fn stored_file_path(workspace_dir: &Path, file: &tiny_artifacts::FileRef) -> PathBuf {
    version_files_dir(workspace_dir).join(&file.key)
}
