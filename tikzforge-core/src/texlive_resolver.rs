use std::path::{Path, PathBuf};

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
pub fn bundled_engine_path(resource_dir: &Path, platform: Platform) -> PathBuf {
    let exe_name = match platform {
        Platform::Windows => "tectonic.exe",
        _ => "tectonic",
    };
    resource_dir.join("tectonic").join(exe_name)
}

/// Dev-mode fallback locations for the staged engine.
///
/// `tauri dev` never bundles resources (on Linux the resource dir resolves
/// to `<exe_dir>/../lib/<app>`, which holds no engine). Developers stage one
/// via `scripts/prepare-texlive.sh` at `src-tauri/tectonic/`, so probe for it
/// relative to the running executable (covers `src-tauri/target/debug/<exe>`)
/// and relative to the working directory (covers repo root / `src-tauri/`).
pub fn dev_staged_candidates(
    exe_dir: &std::path::Path,
    cwd: &std::path::Path,
    platform: Platform,
) -> Vec<PathBuf> {
    let exe_name: &str = match platform {
        Platform::Windows => "tectonic.exe",
        _ => "tectonic",
    };
    let mut out = Vec::new();
    let mut dir = Some(exe_dir);
    for _ in 0..4 {
        let Some(d) = dir else { break };
        out.push(d.join("tectonic").join(exe_name));
        dir = d.parent();
    }
    out.push(cwd.join("src-tauri").join("tectonic").join(exe_name));
    out.push(cwd.join("tectonic").join(exe_name));
    out
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
///
/// `resource_dir` is the Tauri (or other host) resource directory, if
/// available. When `None`, bundled-resource probing is skipped.
pub fn resolve_engine(resource_dir: Option<PathBuf>) -> Engine {
    let platform = current_platform();
    if is_bundled(platform) {
        if let Some(rd) = resource_dir {
            let path = bundled_engine_path(&rd, platform);
            if path.is_file() {
                // Best-effort: bundlers may strip the Unix exec bit.
                let _ = ensure_executable(&path);
                return Engine { path };
            }
        }
        // `tauri dev` never bundles resources, so probe the source-tree
        // staged location (`scripts/prepare-texlive.sh` output) before
        // falling back to PATH.
        let exe_dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(PathBuf::from))
            .unwrap_or_default();
        let cwd = std::env::current_dir().unwrap_or_default();
        for path in dev_staged_candidates(&exe_dir, &cwd, platform) {
            if path.is_file() {
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
