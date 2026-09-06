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
pub fn bundled_engine_path(resource_dir: &PathBuf, platform: Platform) -> PathBuf {
    let exe_name = match platform {
        Platform::Windows => "tectonic.exe",
        _ => "tectonic",
    };
    resource_dir.join("tectonic").join(exe_name)
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

/// Resolve which engine to use: bundled if present, else fall back to PATH.
pub fn resolve_engine(handle: &tauri::AppHandle) -> Engine {
    use tauri::Manager;
    let platform = current_platform();
    if is_bundled(platform) {
        if let Ok(resource_dir) = handle.path().resource_dir() {
            let path = bundled_engine_path(&resource_dir.into(), platform);
            if path.is_file() {
                return Engine { path };
            }
        }
    }
    // Fallback for dev / non-bundled builds.
    let exe_name = match platform {
        Platform::Windows => "tectonic.exe",
        _ => "tectonic",
    };
    Engine {
        path: exe_name.into(),
    }
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
}
