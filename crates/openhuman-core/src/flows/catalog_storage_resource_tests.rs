use super::*;

use crate::core::runtime::{ContextOverlay, CoreContext, DomainSet};
use crate::tools::toolpacks::ToolGroups;

const WORKSPACE_COUNT: usize = 8;

fn context(config: &Config, profile: &str) -> DispatchCtx {
    let root = CoreContext::for_test_with_config(DomainSet::full(), config.clone());
    DispatchCtx::new(
        root.derive_with(
            ContextOverlay::new(config.clone(), DomainSet::full(), ToolGroups::default())
                .profile(profile),
        ),
    )
}

fn open_workspace_fd_count(workspaces: &[tempfile::TempDir]) -> usize {
    use std::collections::HashSet;
    use std::os::unix::fs::MetadataExt;

    // Other tests legitimately open unrelated files in this process. Compare
    // file identities under our temporary workspaces, not the process total.
    let files: HashSet<_> = workspaces
        .iter()
        .flat_map(|workspace| walkdir::WalkDir::new(workspace.path()))
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .filter_map(|entry| entry.metadata().ok())
        .map(|meta| (meta.dev(), meta.ino()))
        .collect();
    std::fs::read_dir("/dev/fd")
        .expect("Unix exposes the current process descriptor directory")
        .filter_map(Result::ok)
        .filter_map(|entry| std::fs::metadata(entry.path()).ok())
        .filter(|meta| files.contains(&(meta.dev(), meta.ino())))
        .count()
}

#[tokio::test]
async fn workspace_fd_lifetime_is_separate_from_catalog_callback_lifetime() {
    // Warm the process-wide bus before measuring. Serving additional callback
    // objects should only add entries to that shared bus, not open a backend.
    crate::modules::runtime::runtime()
        .await
        .expect("process module runtime starts");
    let workspaces: Vec<_> = (0..WORKSPACE_COUNT)
        .map(|_| tempfile::tempdir().expect("isolated workspace"))
        .collect();
    let before = open_workspace_fd_count(&workspaces);
    let mut registrations = Vec::with_capacity(WORKSPACE_COUNT);
    for (index, workspace) in workspaces.iter().enumerate() {
        let config = Config {
            workspace_dir: workspace.path().to_path_buf(),
            ..Config::default()
        };
        let context = context(&config, &format!("fd-probe-{index}"));
        registrations.push(
            register(&context, &config)
                .await
                .expect("scope callback registers without opening a workspace backend"),
        );
    }
    let after_registrations = open_workspace_fd_count(&workspaces);
    assert_eq!(
        after_registrations, before,
        "idle callback registration opened workspace descriptors"
    );

    // The existing default storage cache intentionally keeps a backend per
    // workspace. Exercise that separate owner, then use its existing release
    // API and retain TempDirs until every cached backend has been dropped.
    for workspace in &workspaces {
        let backend = crate::storage::local::backend_for(workspace.path())
            .expect("open the temporary workspace default backend");
        assert_eq!(backend.driver(), "sqlite");
        drop(backend);
    }
    let after_backend_opens = open_workspace_fd_count(&workspaces);
    assert!(
        after_backend_opens >= after_registrations + WORKSPACE_COUNT,
        "opening one SQLite database per workspace should retain file descriptors: registrations={after_registrations}, backends={after_backend_opens}"
    );

    for workspace in &workspaces {
        crate::storage::local::forget(workspace.path());
    }
    drop(registrations);
    let after_release = open_workspace_fd_count(&workspaces);
    eprintln!(
        "workspace_fd_lifetime before={before} after_registrations={after_registrations} after_backend_opens={after_backend_opens} after_release={after_release} workspaces={WORKSPACE_COUNT}"
    );
    assert_eq!(
        after_release, after_registrations,
        "forgetting cached workspace backends should release their descriptors"
    );
}
