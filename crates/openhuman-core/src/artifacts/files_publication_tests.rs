use super::*;

#[test]
fn publication_parent_open_rejects_a_root_swapped_for_a_link() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("Files");
    let outside = temp.path().join("outside");
    std::fs::create_dir(&root).unwrap();
    std::fs::create_dir(&outside).unwrap();
    let checked = root.canonicalize().unwrap();
    std::fs::rename(&root, temp.path().join("previous")).unwrap();
    link_directory(&outside, &root);
    assert!(open_projection_parent(&checked).is_err());
    assert_eq!(std::fs::read_dir(outside).unwrap().count(), 0);
}

#[cfg(unix)]
fn link_directory(target: &Path, link: &Path) {
    std::os::unix::fs::symlink(target, link).unwrap();
}

#[cfg(windows)]
fn link_directory(target: &Path, link: &Path) {
    use base64::Engine;
    use std::os::windows::process::CommandExt;
    // Junctions do not require Windows' optional symlink privilege.
    let literal = |path: &Path| path.to_string_lossy().replace('\'', "''");
    let script = format!(
        "New-Item -ItemType Junction -Path '{}' -Target '{}' -ErrorAction Stop | Out-Null",
        literal(link),
        literal(target)
    );
    let encoded = base64::engine::general_purpose::STANDARD.encode(
        script
            .encode_utf16()
            .flat_map(u16::to_le_bytes)
            .collect::<Vec<_>>(),
    );
    let result = std::process::Command::new("powershell.exe")
        .args(["-NoProfile", "-NonInteractive", "-EncodedCommand", &encoded])
        .creation_flags(0x0800_0000)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
}

#[tokio::test]
async fn publication_replaces_a_hardlink_without_changing_the_linked_file() {
    let temp = tempfile::tempdir().unwrap();
    let files = temp.path().join("Files");
    std::fs::create_dir(&files).unwrap();
    let outside = temp.path().join("outside.txt");
    std::fs::write(&outside, b"protected original").unwrap();
    let destination = files.join("result.txt");
    std::fs::hard_link(&outside, &destination).unwrap();
    let roots = FileRoots::new(files);
    write_visible_projection(
        &crate::core::runtime::DispatchCtx::unbound(),
        &roots,
        &destination,
        b"published result",
    )
    .await
    .unwrap();
    assert_eq!(std::fs::read(&outside).unwrap(), b"protected original");
    assert_eq!(std::fs::read(&destination).unwrap(), b"published result");
}

#[tokio::test]
async fn publication_does_not_write_through_a_destination_swapped_after_validation() {
    let temp = tempfile::tempdir().unwrap();
    let files = temp.path().join("Files");
    std::fs::create_dir(&files).unwrap();
    let destination = files.join("result.txt");
    std::fs::write(&destination, b"placeholder").unwrap();
    let resolved = resolve_visible_projection_destination(&FileRoots::new(files), &destination)
        .await
        .unwrap();
    let outside = temp.path().join("outside.txt");
    std::fs::write(&outside, b"unchanged").unwrap();
    std::fs::remove_file(&destination).unwrap();
    std::fs::hard_link(&outside, &destination).unwrap();
    let (staged, mut file) = StagedProjection::create(resolved).unwrap();
    file.write_all(b"safe replacement").await.unwrap();
    staged
        .publish(&crate::core::runtime::DispatchCtx::unbound(), file)
        .await
        .unwrap();
    assert_eq!(std::fs::read(&outside).unwrap(), b"unchanged");
    assert_eq!(std::fs::read(&destination).unwrap(), b"safe replacement");
}
