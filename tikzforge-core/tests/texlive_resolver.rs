use std::path::PathBuf;
use tikzforge_core::texlive_resolver::*;

#[test]
fn is_bundled_linux() {
    assert!(is_bundled(Platform::Linux));
}

#[test]
fn is_bundled_windows() {
    assert!(is_bundled(Platform::Windows));
}

#[test]
fn is_bundled_macos_not_in_v01() {
    assert!(!is_bundled(Platform::MacOS));
}

#[test]
fn bundled_path_linux() {
    let dir = PathBuf::from("/app/resources");
    let path = bundled_engine_path(&dir, Platform::Linux);
    assert_eq!(path, PathBuf::from("/app/resources/tectonic/tectonic"));
}

#[test]
fn bundled_path_windows() {
    let dir = PathBuf::from("C:\\resources");
    let path = bundled_engine_path(&dir, Platform::Windows);
    assert!(path.ends_with("tectonic/tectonic.exe") || path.ends_with("tectonic\\tectonic.exe"));
}

#[test]
fn current_platform_is_linux_on_linux() {
    if cfg!(target_os = "linux") {
        assert_eq!(current_platform(), Platform::Linux);
    }
}

#[test]
fn ensure_executable_sets_exec_bit() {
    let path =
        std::env::temp_dir().join(format!("tikzforge-ensure-exec-test-{}", std::process::id()));
    std::fs::write(&path, "stub").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).unwrap();
    }
    ensure_executable(&path).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(&path).unwrap().permissions().mode();
        assert!(mode & 0o111 != 0, "exec bit missing after ensure: {mode:o}");
    }
    std::fs::remove_file(&path).ok();
}

#[test]
fn ensure_executable_missing_file_errors() {
    let path = std::env::temp_dir().join(format!(
        "tikzforge-ensure-exec-missing-{}",
        std::process::id()
    ));
    std::fs::remove_file(&path).ok();
    #[cfg(unix)]
    assert!(ensure_executable(&path).is_err());
    #[cfg(not(unix))]
    assert!(ensure_executable(&path).is_ok());
}

#[test]
fn dev_staged_candidates_find_src_tauri_tree_from_target_debug() {
    let exe_dir = PathBuf::from("/repo/src-tauri/target/debug");
    let cwd = PathBuf::from("/repo");
    let candidates = dev_staged_candidates(&exe_dir, &cwd, Platform::Linux);
    assert!(
        candidates.contains(&PathBuf::from("/repo/src-tauri/tectonic/tectonic")),
        "expected src-tauri staged path in {candidates:?}"
    );
}

#[test]
fn dev_staged_candidates_cover_cwd_layouts() {
    let exe_dir = PathBuf::from("/nowhere/bin");
    let cwd = PathBuf::from("/repo");
    let candidates = dev_staged_candidates(&exe_dir, &cwd, Platform::Linux);
    assert!(candidates.contains(&PathBuf::from("/repo/src-tauri/tectonic/tectonic")));
    assert!(candidates.contains(&PathBuf::from("/repo/tectonic/tectonic")));
}

#[test]
fn dev_staged_candidates_windows_uses_exe() {
    let exe_dir = PathBuf::from("C:\\repo\\src-tauri\\target\\debug");
    let cwd = PathBuf::from("C:\\repo");
    let candidates = dev_staged_candidates(&exe_dir, &cwd, Platform::Windows);
    assert!(candidates.iter().all(|p| p.ends_with("tectonic.exe")));
    assert!(!candidates.is_empty());
}
