use std::path::PathBuf;

/// Platform + architecture pair that determines which bundled TeX Live to use.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    Linux,
    Windows,
    MacOS,
}

/// Returns true if TikzForge ships a bundled TeX Live for this platform.
/// v0.1 bundles only Linux (x86_64) and Windows (x86_64).
pub fn is_bundled(platform: Platform) -> bool {
    matches!(platform, Platform::Linux | Platform::Windows)
}

/// Pure resolver: given the Tauri resource directory and a platform, return
/// the absolute path where the bundled pdflatex binary is expected to live.
///
/// Layout: `<resource_dir>/texlive/<platform-bin-dir>/pdflatex[.exe]`
pub fn bundled_pdflatex_path(resource_dir: &PathBuf, platform: Platform) -> PathBuf {
    let (bin_dir, exe_name) = match platform {
        Platform::Linux => ("linux/bin/x86_64-linux", "pdflatex"),
        Platform::Windows => ("windows/bin/win32", "pdflatex.exe"),
        Platform::MacOS => ("macos/bin/universal-darwin", "pdflatex"),
    };
    resource_dir
        .join("texlive")
        .join(bin_dir)
        .join(exe_name)
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
        let path = bundled_pdflatex_path(&dir, Platform::Linux);
        assert_eq!(
            path,
            Path::new("/app/resources/texlive/linux/bin/x86_64-linux/pdflatex")
        );
    }

    #[test]
    fn bundled_path_windows() {
        let dir = PathBuf::from("C:\\resources");
        let path = bundled_pdflatex_path(&dir, Platform::Windows);
        // On Unix, "C:\resources" is a single filename component; just verify
        // the tail matches the expected TeX Live layout.
        let expected_suffix = ["texlive", "windows", "bin", "win32", "pdflatex.exe"];
        let tail: Vec<_> = path.components().rev().take(5).collect();
        let tail: Vec<_> = tail.into_iter().rev().map(|c| c.as_os_str()).collect();
        assert_eq!(tail, expected_suffix);
        assert!(path.ends_with("texlive/windows/bin/win32/pdflatex.exe"));
    }

    #[test]
    fn bundled_path_macos() {
        let dir = PathBuf::from("/app/resources");
        let path = bundled_pdflatex_path(&dir, Platform::MacOS);
        assert_eq!(
            path,
            Path::new("/app/resources/texlive/macos/bin/universal-darwin/pdflatex")
        );
    }

    #[test]
    fn current_platform_is_linux_on_linux() {
        // This test only verifies the function returns the cfg-appropriate value.
        // On a Linux CI runner this must be Platform::Linux.
        if cfg!(target_os = "linux") {
            assert_eq!(current_platform(), Platform::Linux);
        }
    }
}
