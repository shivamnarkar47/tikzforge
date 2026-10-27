use flate2::read::GzDecoder;
use serde::Serialize;
use std::fs;
use std::io::{BufRead, BufReader, Read};
use std::path::PathBuf;
use std::process::{Child, Command, ExitStatus, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

/// Tauri event carrying one engine output line while a compile runs.
/// (Hosts may forward channel lines under their own event name.)
pub const COMPILE_LOG_CHANNEL: &str = "compile-log";

/// How long a single Tectonic run may take before it is killed.
const COMPILE_TIMEOUT: Duration = Duration::from_secs(10 * 60);

/// Monotonic compile generation. Every `compile_tex` start and every
/// `cancel_compile` bumps it.
pub static COMPILE_GEN: AtomicU64 = AtomicU64::new(0);

/// Outcome of running the engine process with a timeout.
#[derive(Debug)]
pub enum RunOutcome {
    Finished(ExitStatus),
    TimedOut,
    Cancelled,
}

/// Wait for `child` (whose pipes are drained elsewhere), killing it if
/// `timeout` elapses or a newer compile generation supersedes `my_gen`.
pub fn run_with_timeout(
    child: &mut Child,
    timeout: Duration,
    current_gen: &AtomicU64,
    my_gen: u64,
) -> std::io::Result<RunOutcome> {
    let start = Instant::now();
    loop {
        if let Some(status) = child.try_wait()? {
            return Ok(RunOutcome::Finished(status));
        }
        if current_gen.load(Ordering::SeqCst) != my_gen {
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
/// shared transcript and forward it via `log_tx`.
fn spawn_log_streamer<R>(
    stream: Option<R>,
    transcript: Arc<Mutex<String>>,
    log_tx: Option<mpsc::UnboundedSender<String>>,
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
                        if let Some(tx) = &log_tx {
                            let _ = tx.send(text);
                        }
                    }
                }
            }
        })
    })
}

#[derive(Serialize, Clone)]
pub struct CompileResult {
    pub pdf: Option<Vec<u8>>,
    pub log: String,
    pub success: bool,
}

/// Resolve where to write the .tex file.
pub fn resolve_tex_path(path: &str) -> Result<PathBuf, String> {
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

/// Synchronous compile implementation.
pub fn compile_tex(
    path: String,
    content: String,
    resource_dir: Option<PathBuf>,
    log_tx: Option<mpsc::UnboundedSender<String>>,
) -> Result<CompileResult, String> {
    let my_gen = COMPILE_GEN.fetch_add(1, Ordering::SeqCst) + 1;
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

    let engine = crate::texlive_resolver::resolve_engine(resource_dir);
    let mut child = Command::new(&engine.path)
        .arg("--synctex")
        .arg(&file_name)
        .current_dir(&dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to run Tectonic: {e}"))?;

    let transcript = Arc::new(Mutex::new(String::new()));
    let out_reader = spawn_log_streamer(
        child.stdout.take(),
        Arc::clone(&transcript),
        log_tx.clone(),
    );
    let err_reader = spawn_log_streamer(
        child.stderr.take(),
        Arc::clone(&transcript),
        log_tx,
    );

    let outcome = run_with_timeout(&mut child, COMPILE_TIMEOUT, &COMPILE_GEN, my_gen)
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

/// Request cancellation of the running compile.
pub fn cancel_compile() {
    COMPILE_GEN.fetch_add(1, Ordering::SeqCst);
}

/// Async wrapper exposing the compile via tokio channels.
pub async fn compile_tex_async(
    path: String,
    content: String,
    resource_dir: Option<PathBuf>,
    log_tx: Option<mpsc::UnboundedSender<String>>,
) -> Result<CompileResult, String> {
    tokio::task::spawn_blocking(move || compile_tex(path, content, resource_dir, log_tx))
        .await
        .map_err(|e| format!("compile task failed: {e}"))?
}

/// Decompress and read a .synctex.gz file.
pub fn read_synctex(path: &str) -> Result<String, String> {
    let bytes = fs::read(path).map_err(|e| e.to_string())?;
    let mut decoder = GzDecoder::new(&bytes[..]);
    let mut content = String::new();
    decoder
        .read_to_string(&mut content)
        .map_err(|e| format!("Failed to decompress {}: {e}", PathBuf::from(path).display()))?;
    Ok(content)
}