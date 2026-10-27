use std::path::PathBuf;
use tauri::Emitter;
use tauri::Manager;

#[tauri::command]
async fn compile_tex(
    handle: tauri::AppHandle,
    path: String,
    content: String,
) -> Result<tikzforge_core::compile::CompileResult, String> {
    let resource_dir = handle.path().resource_dir().ok().map(PathBuf::from);
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<String>();

    let handle_clone = handle.clone();
    let forwarder = tokio::spawn(async move {
        while let Some(line) = rx.recv().await {
            let _ = handle_clone.emit("compile-log", line);
        }
    });

    let result = tikzforge_core::compile::compile_tex_async(path, content, resource_dir, Some(tx)).await;
    let _ = forwarder.await;
    result
}

#[tauri::command]
fn cancel_compile() {
    tikzforge_core::compile::cancel_compile();
}

#[tauri::command]
fn detect_engine(handle: tauri::AppHandle) -> Option<String> {
    let resource_dir = handle.path().resource_dir().ok().map(PathBuf::from);
    let engine = tikzforge_core::texlive_resolver::resolve_engine(resource_dir);
    if engine.path.is_file() {
        Some(engine.path.to_string_lossy().into_owned())
    } else {
        None
    }
}

#[tauri::command]
fn read_synctex(path: String) -> Result<String, String> {
    tikzforge_core::compile::read_synctex(&path)
}

#[derive(serde::Serialize)]
struct UpdateInfo {
    available: bool,
}

#[tauri::command]
fn check_update() -> UpdateInfo {
    UpdateInfo { available: false }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            compile_tex,
            cancel_compile,
            detect_engine,
            read_synctex,
            check_update
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}