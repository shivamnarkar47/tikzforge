mod texlive_resolver;

use flate2::read::GzDecoder;
use serde::Serialize;
use std::fs;
use std::io::Read;
use std::path::PathBuf;
use std::process::Command;

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

/// Resolve which engine to use. Prefers the bundled engine (Tectonic on
/// Windows, pdflatex on Linux), then falls back to PATH.
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
    let output = match &engine {
        texlive_resolver::Engine::Tectonic(tectonic) => {
            // Tectonic is self-contained: it handles its own dependency
            // downloads and produces a single-pass PDF. No -synctex needed.
            Command::new(tectonic)
                .arg(&file_name)
                .current_dir(&dir)
                .output()
        }
        texlive_resolver::Engine::PdfLatex(pdflatex) => {
            Command::new(pdflatex)
                .args(["-interaction=nonstopmode", "-shell-escape", "-synctex=1"])
                .arg(&file_name)
                .current_dir(&dir)
                .output()
        }
    }
    .map_err(|e| format!("Failed to run engine: {e}"))?;

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
    match resolve_engine(&handle) {
        texlive_resolver::Engine::Tectonic(path)
        | texlive_resolver::Engine::PdfLatex(path) => {
            if path.is_file() {
                Some(path.to_string_lossy().into_owned())
            } else {
                None
            }
        }
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
