mod texlive_resolver;

use flate2::read::GzDecoder;
use serde::Serialize;
use std::fs;
use std::io::{BufRead, BufReader, Read};
use std::path::PathBuf;
use std::process::{Child, Command, ExitStatus, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tauri::Emitter;

/// Tauri event carrying one engine output line while a compile runs.
const COMPILE_LOG_EVENT: &str = "compile-log";

/// How long a single Tectonic run may take before it is killed.
/// First-ever compiles download the engine bundle and fonts (minutes on slow
/// links); normal documents finish in seconds. Without a ceiling a stalled
/// download blocks the UI spinner forever.
const COMPILE_TIMEOUT: Duration = Duration::from_secs(10 * 60);

/// Set by `cancel_compile`; the running `compile_tex` poll loop observes it
/// within milliseconds and kills its child process.
static COMPILE_CANCEL: AtomicBool = AtomicBool::new(false);

/// Outcome of running the engine process with a timeout.
#[derive(Debug)]
enum RunOutcome {
    Finished(ExitStatus),
    TimedOut,
    Cancelled,
}

/// Wait for `child` (whose pipes are drained elsewhere), killing it if
/// `timeout` elapses or `cancel` is set. Always reaps the process.
fn run_with_timeout(
    child: &mut Child,
    timeout: Duration,
    cancel: &AtomicBool,
) -> std::io::Result<RunOutcome> {
    let start = Instant::now();
    loop {
        if let Some(status) = child.try_wait()? {
            return Ok(RunOutcome::Finished(status));
        }
        if cancel.load(Ordering::SeqCst) {
            let _ = child.kill();
            child.wait()?;
            return Ok(RunOutcome::Cancelled);
        }
        if start.elapsed() >= timeout {
            let _ = child.kill();
            child.wait()?;
            return Ok(RunOutcome::TimedOut);
        }
        std::thread::sleep(Duration::from_millis(50));
    }
}

/// Drain one engine pipe on a background thread: append every line to the
/// shared transcript and forward it to the frontend as a `compile-log`
/// event. Returns the thread handle so the caller can join it (pipes hit
/// EOF once the child exits) before reading the finished transcript.
fn spawn_log_streamer<R>(
    stream: Option<R>,
    handle: tauri::AppHandle,
    transcript: Arc<Mutex<String>>,
) -> Option<std::thread::JoinHandle<()>>
where
    R: Read + Send + 'static,
{
    stream.map(|s| {
        std::thread::spawn(move || {
            let mut reader = BufReader::new(s);
            let mut line = String::new();
            loop {
                line.clear();
                match reader.read_line(&mut line) {
                    Ok(0) | Err(_) => break,
                    Ok(_) => {
                        let text = line.trim_end().to_owned();
                        if let Ok(mut t) = transcript.lock() {
                            t.push_str(&text);
                            t.push('\n');
                        }
                        let _ = handle.emit(COMPILE_LOG_EVENT, text);
                    }
                }
            }
        })
    })
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
    // A new compile supersedes any previous one still running.
    COMPILE_CANCEL.store(false, Ordering::SeqCst);
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
    let mut child = Command::new(&engine.path)
        .arg(&file_name)
        .current_dir(&dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to run Tectonic: {e}"))?;

    // Stream both pipes live: each line goes to the shared transcript and to
    // the frontend as a `compile-log` event. Join the threads (EOF follows
    // child exit) before reading the finished transcript.
    let transcript = Arc::new(Mutex::new(String::new()));
    let out_reader = spawn_log_streamer(
        child.stdout.take(),
        handle.clone(),
        Arc::clone(&transcript),
    );
    let err_reader = spawn_log_streamer(
        child.stderr.take(),
        handle.clone(),
        Arc::clone(&transcript),
    );

    let outcome = run_with_timeout(&mut child, COMPILE_TIMEOUT, &COMPILE_CANCEL)
        .map_err(|e| format!("Failed while running Tectonic: {e}"))?;
    for reader in [out_reader, err_reader].into_iter().flatten() {
        let _ = reader.join();
    }
    let log = transcript.lock().map(|t| t.clone()).unwrap_or_default();

    let stem = tex_path
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "untitled".to_string());

    match outcome {
        RunOutcome::Finished(status) => {
            let pdf = fs::read(dir.join(format!("{stem}.pdf"))).ok();
            let success = status.success() && pdf.is_some();
            Ok(CompileResult { pdf, log, success })
        }
        RunOutcome::TimedOut => Ok(CompileResult {
            pdf: None,
            log: format!(
                "Tectonic did not finish within {} minutes and was stopped.\n\
                 Note: the first compile downloads the engine bundle and fonts, \
                 which can take several minutes on slow connections — retry once \
                 it has cached them.\n\nPartial output:\n{log}",
                COMPILE_TIMEOUT.as_secs() / 60,
            ),
            success: false,
        }),
        RunOutcome::Cancelled => Ok(CompileResult {
            pdf: None,
            log: format!("Compile cancelled by user.\n\nPartial output:\n{log}"),
            success: false,
        }),
    }
}

/// Request cancellation of the running compile, if any. The `compile_tex`
/// poll loop observes the flag within milliseconds, kills Tectonic, and
/// returns a "cancelled" result. Fire-and-forget safe: setting the flag with
/// no compile running is a no-op (the next `compile_tex` clears it on start).
#[tauri::command]
fn cancel_compile() {
    COMPILE_CANCEL.store(true, Ordering::SeqCst);
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
            cancel_compile,
            detect_engine,
            read_synctex,
            check_update
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
