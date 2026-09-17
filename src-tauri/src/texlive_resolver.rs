use std::path::PathBuf;

/// Platform that determines which bundled engine to use.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    Linux,
    Windows,
    MacOS,
}

/// The bundled engine: Tectonic is self-contained on all platforms.
pub struct Engine {
    pub path: PathBuf,
}

/// Pure resolver: given the Tauri resource directory and a platform, return
/// the absolute path where the bundled Tectonic binary is expected to live.
///
/// All platforms: `<resource_dir>/tectonic/tectonic[.exe]`
/// (Tauri's resource bundler preserves the relative layout of `tectonic/*`.)
pub fn bundled_engine_path(resource_dir: &PathBuf, platform: Platform) -> PathBuf {
    let exe_name = match platform {
        Platform::Windows => "tectonic.exe",
        _ => "tectonic",
    };
    resource_dir.join("tectonic").join(exe_name)
}

/// Ensure the staged binary is executable.
///
/// Tauri's resource bundlers do not always preserve the Unix exec bit inside
/// AppImage/deb packages. A detected-but-unexecutable engine would fail at
/// spawn time with "permission denied", so repair it best-effort at resolve
/// time. No-op on Windows.
pub fn ensure_executable(path: &std::path::Path) -> std::io::Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(path)?.permissions();
        if perms.mode() & 0o111 == 0 {
            perms.set_mode(perms.mode() | 0o755);
            std::fs::set_permissions(path, perms)?;
        }
    }
    #[cfg(not(unix))]
    {
        let _ = path;
    }
    Ok(())
}

/// Returns true if TikzForge ships a bundled engine for this platform.
pub fn is_bundled(platform: Platform) -> bool {
    matches!(platform, Platform::Linux | Platform::Windows)
}

/// Detect the current platform at runtime.
pub fn current_platform() -> Platform {
    if cfg!(target_os = "windows") {
        Platform::Windows
    } else if cfg!(target_os = "macos") {
        Platform::MacOS
    } else {
        Platform::Linux
    }
}

/// Search each directory in `PATH` for `name` and return the first match.
fn find_on_path(name: &str) -> Option<PathBuf> {
    std::env::var_os("PATH").and_then(|path_var| {
        std::env::split_paths(&path_var).find_map(|dir| {
            let candidate = dir.join(name);
            if candidate.is_file() {
                Some(candidate)
            } else {
                None
            }
        })
    })
}

/// Resolve which engine to use: bundled if present, else fall back to PATH.
pub fn resolve_engine(handle: &tauri::AppHandle) -> Engine {
    use tauri::Manager;
    let platform = current_platform();
    if is_bundled(platform) {
        if let Ok(resource_dir) = handle.path().resource_dir() {
            let path = bundled_engine_path(&resource_dir.into(), platform);
            if path.is_file() {
                // Best-effort: bundlers may strip the Unix exec bit.
                let _ = ensure_executable(&path);
                return Engine { path };
            }
        }
    }
    // Fallback for dev / non-bundled builds: resolve absolute path via PATH
    // so that detect_engine's is_file() check works correctly.
    let exe_name = match platform {
        Platform::Windows => "tectonic.exe",
        _ => "tectonic",
    };
    let path = find_on_path(exe_name).unwrap_or_else(|| PathBuf::from(exe_name));
    Engine { path }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

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
        assert_eq!(path, Path::new("/app/resources/tectonic/tectonic"));
    }

    #[test]
    fn bundled_path_windows() {
        let dir = PathBuf::from("C:\\resources");
        let path = bundled_engine_path(&dir, Platform::Windows);
        let expected_suffix = ["tectonic", "tectonic.exe"];
        let tail: Vec<_> = path.components().rev().take(2).collect();
        let tail: Vec<_> = tail.into_iter().rev().map(|c| c.as_os_str()).collect();
        assert_eq!(tail, expected_suffix);
        assert!(path.ends_with("tectonic/tectonic.exe"));
    }

    #[test]
    fn current_platform_is_linux_on_linux() {
        if cfg!(target_os = "linux") {
            assert_eq!(current_platform(), Platform::Linux);
        }
    }

    #[test]
    fn ensure_executable_sets_exec_bit() {
        let path = std::env::temp_dir().join(format!(
            "tikzforge-ensure-exec-test-{}",
            std::process::id()
        ));
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
}
