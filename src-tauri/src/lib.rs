mod texlive_resolver;

use flate2::read::GzDecoder;
use serde::Serialize;
use std::fs;
use std::io::Read;
use std::path::PathBuf;
use std::process::{Child, Command, Output, Stdio};
use std::time::{Duration, Instant};

/// How long a single Tectonic run may take before it is killed.
/// First-ever compiles download the engine bundle and fonts (minutes on slow
/// links); normal documents finish in seconds. Without a ceiling a stalled
/// download blocks the UI spinner forever.
const COMPILE_TIMEOUT: Duration = Duration::from_secs(10 * 60);

/// Outcome of running the engine process with a timeout.
#[derive(Debug)]
enum RunOutcome {
    Finished(Output),
    /// The process was killed after the timeout; carries partial output.
    TimedOut(Output),
}

/// Wait for `child`, killing it if `timeout` elapses. Always reaps the
/// process and returns whatever output it produced.
fn run_with_timeout(mut child: Child, timeout: Duration) -> std::io::Result<RunOutcome> {
    let start = Instant::now();
    loop {
        match child.try_wait()? {
            Some(_) => return child.wait_with_output().map(RunOutcome::Finished),
            None if start.elapsed() >= timeout => {
                let _ = child.kill();
                return child.wait_with_output().map(RunOutcome::TimedOut);
            }
            None => std::thread::sleep(Duration::from_millis(50)),
        }
    }
}

#[derive(Serialize)]
struct CompileResult {
    pdf: Option<Vec<u8>>,
    log: String,
    success: bool,
}

/// Resolve where to write the .tex file. Absolute paths are used as-is;
/// bare filenames go into a scratch dir so browser-style names like
/// "untitled.tex" still compile.
fn resolve_tex_path(path: &str) -> Result<PathBuf, String> {
    let candidate = PathBuf::from(path);
    if candidate.is_absolute() {
        if let Some(parent) = candidate.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
        }
        return Ok(candidate);
    }
    let dir = std::env::temp_dir().join("tikzforge");
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let file_name = candidate
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .filter(|n| !n.is_empty())
        .unwrap_or_else(|| "untitled.tex".to_string());
    Ok(dir.join(file_name))
}

/// Resolve which engine to use. Tectonic on all platforms.
fn resolve_engine(handle: &tauri::AppHandle) -> texlive_resolver::Engine {
    texlive_resolver::resolve_engine(handle)
}

#[tauri::command]
fn compile_tex(handle: tauri::AppHandle, path: String, content: String) -> Result<CompileResult, String> {
    let tex_path = resolve_tex_path(&path)?;
    fs::write(&tex_path, content).map_err(|e| e.to_string())?;

    let dir = tex_path
        .parent()
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::temp_dir());
    let file_name = tex_path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "untitled.tex".to_string());

    let engine = resolve_engine(&handle);
    // Tectonic is self-contained: handles its own dependency downloads and
    // produces a single-pass PDF. No -synctex or -shell-escape flags needed.
    let child = Command::new(&engine.path)
        .arg(&file_name)
        .current_dir(&dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to run Tectonic: {e}"))?;

    let output = match run_with_timeout(child, COMPILE_TIMEOUT)
        .map_err(|e| format!("Failed while running Tectonic: {e}"))?
    {
        RunOutcome::Finished(output) => output,
        RunOutcome::TimedOut(partial) => {
            let partial_log = String::from_utf8_lossy(&partial.stdout);
            return Ok(CompileResult {
                pdf: None,
                log: format!(
                    "Tectonic did not finish within {} minutes and was stopped.\n\
                     Note: the first compile downloads the engine bundle and fonts, \
                     which can take several minutes on slow connections — retry once \
                     it has cached them.\n\nPartial output:\n{partial_log}",
                    COMPILE_TIMEOUT.as_secs() / 60,
                ),
                success: false,
            });
        }
    };

    let log = String::from_utf8_lossy(&output.stdout).into_owned();
    let stem = tex_path
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "untitled".to_string());
    let pdf = fs::read(dir.join(format!("{stem}.pdf"))).ok();
    let success = output.status.success() && pdf.is_some();

    Ok(CompileResult { pdf, log, success })
}

/// Report the bundled engine path if it exists, else fall back to PATH.
#[tauri::command]
fn detect_engine(handle: tauri::AppHandle) -> Option<String> {
    let engine = resolve_engine(&handle);
    if engine.path.is_file() {
        Some(engine.path.to_string_lossy().into_owned())
    } else {
        None
    }
}

/// Decompress and read a .synctex.gz file for forward SyncTeX.
#[tauri::command]
fn read_synctex(path: String) -> Result<String, String> {
    let bytes = fs::read(&path).map_err(|e| e.to_string())?;
    let mut decoder = GzDecoder::new(&bytes[..]);
    let mut content = String::new();
    decoder.read_to_string(&mut content).map_err(|e| {
        format!("Failed to decompress {}: {e}", PathBuf::from(&path).display())
    })?;
    Ok(content)
}

#[derive(Serialize)]
struct UpdateInfo {
    available: bool,
}

/// Update checks are not wired up yet; report "no update" instead of failing.
#[tauri::command]
fn check_update() -> UpdateInfo {
    UpdateInfo { available: false }
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            compile_tex,
            detect_engine,
            read_synctex,
            check_update
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
