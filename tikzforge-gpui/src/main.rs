#![recursion_limit = "512"]
mod commands;
mod editor;

use editor::Editor;
use editor::EditorEvent;
use gpui::prelude::FluentBuilder;
use gpui::*;
use pdfium_bundled::bind_pdfium_silent;
use pdfium_bundled::pdfium_render::prelude::Pdfium;
use serde::{Deserialize, Serialize};
use smallvec::smallvec;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

// ---------- Synctex ----------

#[derive(Clone, Debug)]
struct SynctexRecord {
    page: i32,
    x: i32,
    y: i32,
    line: i32,
}

fn parse_synctex(content: &str) -> Vec<SynctexRecord> {
    if content.trim().is_empty() {
        return vec![];
    }
    let mut records = vec![];
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('%') {
            continue;
        }
        if !trimmed
            .chars()
            .next()
            .map(|c| c.is_ascii_digit())
            .unwrap_or(false)
        {
            continue;
        }
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.len() >= 5
            && let (Ok(page), Ok(x), Ok(y), Ok(line)) = (
                parts[0].parse::<i32>(),
                parts[1].parse::<i32>(),
                parts[2].parse::<i32>(),
                parts[3].parse::<i32>(),
            )
        {
            records.push(SynctexRecord { page, x, y, line });
        }
    }
    records
}

fn find_source_position(records: &[SynctexRecord], page: i32, x: i32, y: i32) -> Option<i32> {
    let page_records: Vec<&SynctexRecord> = records.iter().filter(|r| r.page == page).collect();
    if page_records.is_empty() {
        return None;
    }
    let mut closest = page_records[0];
    let mut min_dist = (page_records[0].x - x).abs() + (page_records[0].y - y).abs();
    for r in page_records.iter().skip(1) {
        let dist = (r.x - x).abs() + (r.y - y).abs();
        if dist < min_dist {
            min_dist = dist;
            closest = r;
        }
    }
    Some(closest.line)
}

// ---------- Theme ----------

#[derive(Copy, Clone, PartialEq, Eq)]
enum Theme {
    Dark,
    Light,
}

impl Theme {
    fn bg(self) -> u32 {
        match self {
            Theme::Dark => 0x1F1F28,
            Theme::Light => 0xF2ECBC,
        }
    }
    fn fg(self) -> u32 {
        match self {
            Theme::Dark => 0xDCD7BA,
            Theme::Light => 0x545464,
        }
    }
    fn panel(self) -> u32 {
        match self {
            Theme::Dark => 0x2A2A37,
            Theme::Light => 0xE9E3C8,
        }
    }
    fn accent(self) -> u32 {
        match self {
            Theme::Dark => 0x7E9CD8,
            Theme::Light => 0x4C6A8D,
        }
    }
    fn muted(self) -> u32 {
        match self {
            Theme::Dark => 0x727169,
            Theme::Light => 0x8A8980,
        }
    }
}

// ---------- View mode ----------

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum ViewMode {
    Split,
    Editor,
    Preview,
}

impl ViewMode {
    fn label(self) -> &'static str {
        match self {
            ViewMode::Split => "Split",
            ViewMode::Editor => "Editor",
            ViewMode::Preview => "Preview",
        }
    }
}

// ---------- Toast ----------

#[derive(Clone)]
enum ToastKind {
    Success,
    Error,
    Info,
}

#[derive(Clone)]
struct Toast {
    id: u64,
    kind: ToastKind,
    message: String,
}

// ---------- Compile error ----------

#[derive(Clone)]
struct CompileError {
    line: usize,
    message: String,
}

// ---------- Persistence ----------

#[derive(Serialize, Deserialize, Default, Clone)]
struct Persisted {
    dark: bool,
    view_mode: String,
    divider: f32,
    window_x: Option<f32>,
    window_y: Option<f32>,
    window_w: Option<f32>,
    window_h: Option<f32>,
    recent_files: Vec<String>,
    auto_compile: bool,
    log_open: bool,
}

fn config_path() -> PathBuf {
    let base = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    base.join("tikzforge").join("gpui.json")
}

fn load_persisted() -> Persisted {
    let path = config_path();
    if let Some(p) = fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
    {
        p
    } else {
        Persisted {
            dark: true,
            view_mode: "split".to_string(),
            divider: 0.5,
            auto_compile: true,
            ..Default::default()
        }
    }
}

fn save_persisted(p: &Persisted) {
    let path = config_path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Ok(s) = serde_json::to_string_pretty(p) {
        let _ = fs::write(&path, s);
    }
}

// ---------- Document ----------

struct Document {
    path: Option<PathBuf>,
    content: String,
    saved_content: String,
    editor: Entity<Editor>,
    dirty: bool,
}

// ---------- Actions ----------

actions!(
    tikzforge,
    [
        Save,
        Open,
        Compile,
        CommandPalette,
        ToggleTheme,
        ViewSplit,
        ViewEditor,
        ViewPreview,
        QuitApp,
        NewDocument,
        CloseTab
    ]
);

// ---------- App state ----------

struct TikzForgeApp {
    theme: Theme,
    view_mode: ViewMode,
    divider: f32,
    resizing: bool,
    docs: Vec<Document>,
    active: usize,
    status: String,
    focus_handle: FocusHandle,
    window_bounds: Option<Bounds<gpui::Pixels>>,
    pdfium: Option<Pdfium>,
    pdf_bytes: Option<Vec<u8>>,
    page_count: usize,
    current_page: usize,
    zoom: f32,
    fit_width: bool,
    pdf_image: Option<Arc<RenderImage>>,
    pdf_image_dims: Option<(u32, u32)>,
    synctex_records: Vec<SynctexRecord>,
    pdf_viewport_width: f32,
    engine_ok: bool,
    engine_path: PathBuf,
    auto_compile: bool,
    auto_compile_gen: u64,
    compile_busy: bool,
    pending_compile: Option<(String, String)>,
    log_text: String,
    log_open: bool,
    toasts: Vec<Toast>,
    toast_counter: u64,
    recent_files: Vec<String>,
    palette_open: bool,
    palette_query: String,
    palette_selected: usize,
    errors: Vec<CompileError>,
    updater_status: String,
}

const DEFAULT_TEMPLATE: &str = "\\documentclass{standalone}\n\\usepackage{tikz}\n\\begin{document}\n\\begin{tikzpicture}\n  \\draw (0,0) -- (1,1);\n\\end{tikzpicture}\n\\end{document}\n";

impl TikzForgeApp {
    fn new(cx: &mut Context<Self>, persisted: Persisted) -> Self {
        let theme = if persisted.dark {
            Theme::Dark
        } else {
            Theme::Light
        };
        let view_mode = match persisted.view_mode.as_str() {
            "editor" => ViewMode::Editor,
            "preview" => ViewMode::Preview,
            _ => ViewMode::Split,
        };
        let divider = if persisted.divider > 0.0 {
            persisted.divider.clamp(0.2, 0.8)
        } else {
            0.5
        };
        let dark = theme == Theme::Dark;
        let content = DEFAULT_TEMPLATE.to_string();
        let editor = cx.new(|cx| Editor::new(cx, content.clone(), dark));
        let doc = Document {
            path: None,
            content: content.clone(),
            saved_content: content.clone(),
            editor: editor.clone(),
            dirty: false,
        };

        let mut app = Self {
            theme,
            view_mode,
            divider,
            resizing: false,
            docs: vec![doc],
            active: 0,
            status: "Ready".to_string(),
            focus_handle: cx.focus_handle(),
            window_bounds: None,
            pdfium: None,
            pdf_bytes: None,
            page_count: 0,
            current_page: 0,
            zoom: 1.5,
            fit_width: true,
            pdf_image: None,
            pdf_image_dims: None,
            synctex_records: vec![],
            pdf_viewport_width: 800.0,
            engine_ok: true,
            engine_path: PathBuf::new(),
            auto_compile: persisted.auto_compile,
            auto_compile_gen: 0,
            compile_busy: false,
            pending_compile: None,
            log_text: String::new(),
            log_open: persisted.log_open,
            toasts: vec![],
            toast_counter: 0,
            recent_files: persisted.recent_files.clone(),
            palette_open: false,
            palette_query: String::new(),
            palette_selected: 0,
            errors: vec![],
            updater_status: "Idle".to_string(),
        };

        // subscribe to editor changes
        let weak = cx.weak_entity();
        cx.subscribe(&editor, move |_this, _editor, _ev: &EditorEvent, cx| {
            // handled via active doc; call method
            if let Some(this) = weak.upgrade() {
                cx.update_entity(&this, |app, cx| app.on_editor_changed(cx));
            }
        })
        .detach();

        // engine detection
        let engine = tikzforge_core::resolve_engine(None);
        app.engine_ok = engine.path.is_file();
        app.engine_path = engine.path.clone();
        if !app.engine_ok {
            app.status = format!("LaTeX engine not found: {}", engine.path.display());
        }

        app
    }

    fn persist(&self) {
        let (x, y, w, h) = if let Some(b) = self.window_bounds {
            (
                Some(f32::from(b.origin.x)),
                Some(f32::from(b.origin.y)),
                Some(f32::from(b.size.width)),
                Some(f32::from(b.size.height)),
            )
        } else {
            (None, None, None, None)
        };
        let p = Persisted {
            dark: self.theme == Theme::Dark,
            view_mode: match self.view_mode {
                ViewMode::Split => "split".to_string(),
                ViewMode::Editor => "editor".to_string(),
                ViewMode::Preview => "preview".to_string(),
            },
            divider: self.divider,
            window_x: x,
            window_y: y,
            window_w: w,
            window_h: h,
            recent_files: self.recent_files.clone(),
            auto_compile: self.auto_compile,
            log_open: self.log_open,
        };
        save_persisted(&p);
    }

    fn on_editor_changed(&mut self, cx: &mut Context<Self>) {
        self.sync_active(cx);
        if self.auto_compile {
            self.schedule_auto_compile(cx);
        }
        cx.notify();
    }

    fn sync_active(&mut self, cx: &mut Context<Self>) {
        if let Some(doc) = self.docs.get_mut(self.active) {
            let text = doc.editor.read(cx).text();
            doc.content = text.clone();
            doc.dirty = text != doc.saved_content;
        }
    }

    fn active_doc(&self) -> &Document {
        &self.docs[self.active]
    }

    #[allow(dead_code)]
    fn active_doc_mut(&mut self) -> &mut Document {
        &mut self.docs[self.active]
    }

    fn schedule_auto_compile(&mut self, cx: &mut Context<Self>) {
        if !self.auto_compile {
            return;
        }
        self.auto_compile_gen += 1;
        let my_gen = self.auto_compile_gen;
        let (path, content) = self.current_path_content(cx);
        let this = cx.weak_entity();
        gpui::App::spawn(&*cx, async move |cx| {
            Timer::after(Duration::from_secs(2)).await;
            let still = cx
                .update(|app| {
                    if let Some(e) = this.upgrade() {
                        e.read(app).auto_compile_gen == my_gen
                    } else {
                        false
                    }
                })
                .unwrap_or(false);
            if !still {
                return;
            }
            if let Some(e) = this.upgrade() {
                cx.update(|app| {
                    e.update(app, |app, cx| {
                        app.start_compile(path.clone(), content.clone(), cx);
                    });
                })
                .ok();
            }
        })
        .detach();
    }

    fn current_path_content(&self, _cx: &mut Context<Self>) -> (String, String) {
        // sync not needed; use stored content
        let doc = &self.docs[self.active];
        let path = doc
            .path
            .clone()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| "untitled.tex".to_string());
        (path, doc.content.clone())
    }

    fn start_compile(&mut self, path: String, content: String, cx: &mut Context<Self>) {
        if self.compile_busy {
            self.pending_compile = Some((path, content));
            self.status = "Compile queued".to_string();
            cx.notify();
            return;
        }
        self.compile_busy = true;
        self.log_text.clear();
        self.status = "Compiling...".to_string();
        cx.notify();

        let this = cx.weak_entity();
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<String>();
        let bg = cx.background_executor().spawn(async move {
            tikzforge_core::compile_tex(path.clone(), content.clone(), None, Some(tx))
        });

        gpui::App::spawn(&*cx, async move |cx| {
            while let Some(line) = rx.recv().await {
                if let Some(e) = this.upgrade() {
                    e.update(cx, |app, cx| {
                        if !app.log_text.is_empty() {
                            app.log_text.push('\n');
                        }
                        app.log_text.push_str(&line);
                        cx.notify();
                    })
                    .ok();
                }
            }
            let result = bg.await;
            if let Some(e) = this.upgrade() {
                e.update(cx, |app, cx| {
                    app.compile_busy = false;
                    app.apply_compile_result(result, cx);
                    if let Some((p, c)) = app.pending_compile.take() {
                        app.start_compile(p, c, cx);
                    }
                    cx.notify();
                })
                .ok();
            }
        })
        .detach();
    }

    fn apply_compile_result(
        &mut self,
        result: Result<tikzforge_core::CompileResult, String>,
        cx: &mut Context<Self>,
    ) {
        match result {
            Ok(cr) => {
                self.log_text = cr.log.clone();
                if cr.success {
                    if let Some(pdf) = cr.pdf {
                        self.pdf_bytes = Some(pdf);
                        self.current_page = 1;
                        self.render_current_page();
                    }
                    // synctex
                    if let Some(p) = &self.docs[self.active].path {
                        let stem = p
                            .file_stem()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string();
                        let dir = p.parent().unwrap_or_else(|| std::path::Path::new("."));
                        let synctex_path = dir.join(format!("{}.synctex.gz", stem));
                        if let Ok(content) =
                            tikzforge_core::read_synctex(&synctex_path.to_string_lossy())
                        {
                            self.synctex_records = parse_synctex(&content);
                        }
                    }
                    self.status = "Compiled OK".to_string();
                    self.show_toast(ToastKind::Success, "Compiled OK".to_string(), cx);
                } else {
                    self.status = "Compile failed".to_string();
                    self.show_toast(ToastKind::Error, "Compile failed".to_string(), cx);
                }
                self.parse_errors();
            }
            Err(e) => {
                self.status = format!("Compile error: {}", e);
                self.show_toast(ToastKind::Error, format!("Compile error: {}", e), cx);
            }
        }
    }

    fn parse_errors(&mut self) {
        self.errors.clear();
        for line in self.log_text.lines() {
            let lower = line.to_lowercase();
            if lower.contains("error") {
                let mut line_no = 1usize;
                if let Some(idx) = line.find("l.") {
                    let rest = &line[idx + 2..];
                    let num: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
                    if let Ok(n) = num.parse::<usize>() {
                        line_no = n;
                    }
                }
                self.errors.push(CompileError {
                    line: line_no,
                    message: line.trim().to_string(),
                });
            }
        }
    }

    fn compile_current(&mut self, cx: &mut Context<Self>) {
        self.sync_active(cx);
        let (path, content) = self.current_path_content(cx);
        self.start_compile(path, content, cx);
    }

    fn render_current_page(&mut self) {
        if self.pdfium.is_none() {
            self.pdfium = bind_pdfium_silent().ok();
        }
        let pdfium = match &self.pdfium {
            Some(p) => p,
            None => {
                self.status = "PDFium unavailable".to_string();
                return;
            }
        };
        let bytes = match &self.pdf_bytes {
            Some(b) => b.clone(),
            None => return,
        };
        let mut cur = self.current_page;
        if cur < 1 {
            cur = 1;
        }
        let doc = match pdfium.load_pdf_from_byte_slice(&bytes, None) {
            Ok(d) => d,
            Err(e) => {
                self.status = format!("PDF load failed: {}", e);
                return;
            }
        };
        let page_count = doc.pages().len();
        if page_count == 0 {
            self.status = "PDF has no pages".to_string();
            return;
        }
        if cur as i32 > page_count {
            cur = page_count as usize;
        }
        self.current_page = cur;
        let page = match doc.pages().get(cur as i32 - 1) {
            Ok(p) => p,
            Err(e) => {
                self.status = format!("Page error: {}", e);
                return;
            }
        };
        let pw = page.width().value as f32;
        let ph = page.height().value as f32;
        let scale = if self.fit_width {
            let vp = self.pdf_viewport_width.max(200.0);
            (vp / pw.max(1.0)).max(0.25)
        } else {
            self.zoom
        };
        let tw = (pw * scale).round().max(1.0) as i32;
        let th = (ph * scale).round().max(1.0) as i32;
        let bitmap = match page.render(tw, th, None) {
            Ok(b) => b,
            Err(e) => {
                self.status = format!("Render failed: {}", e);
                return;
            }
        };
        let rgba = bitmap.as_rgba_bytes();
        let mut bgra = rgba;
        for chunk in bgra.chunks_exact_mut(4) {
            chunk.swap(0, 2);
        }
        self.page_count = page_count as usize;
        if let Some(buf) = image::RgbaImage::from_raw(tw as u32, th as u32, bgra) {
            let frame = image::Frame::new(buf);
            self.pdf_image = Some(Arc::new(RenderImage::new(smallvec![frame])));
            self.pdf_image_dims = Some((tw as u32, th as u32));
        } else {
            self.status = "Failed to build image buffer".to_string();
        }
    }

    fn save(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        self.save_active(cx);
    }

    fn save_active(&mut self, cx: &mut Context<Self>) {
        self.sync_active(cx);
        let idx = self.active;
        let path_opt = self.docs[idx].path.clone();
        let content = self.docs[idx].content.clone();
        let path = if let Some(p) = path_opt {
            p
        } else {
            match rfd::FileDialog::new()
                .add_filter("TeX", &["tex"])
                .set_file_name("untitled.tex")
                .save_file()
            {
                Some(p) => p,
                None => {
                    cx.notify();
                    return;
                }
            }
        };
        if fs::write(&path, &content).is_ok() {
            let doc = &mut self.docs[idx];
            doc.path = Some(path.clone());
            doc.saved_content = content;
            doc.dirty = false;
            self.status = format!("Saved {}", path.display());
            self.push_recent(path.clone());
            self.show_toast(ToastKind::Success, format!("Saved {}", path.display()), cx);
        } else {
            self.status = "Save failed".to_string();
            self.show_toast(ToastKind::Error, "Save failed".to_string(), cx);
        }
        self.persist();
        cx.notify();
    }

    fn open(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        self.open_document(cx);
    }

    fn open_document(&mut self, cx: &mut Context<Self>) {
        self.sync_active(cx);
        let dialog = rfd::FileDialog::new().add_filter("TeX", &["tex"]);
        if let Some(path) = dialog.pick_file() {
            if let Ok(content) = fs::read_to_string(&path) {
                let dark = self.theme == Theme::Dark;
                let editor = cx.new(|cx| Editor::new(cx, content.clone(), dark));
                let weak = cx.weak_entity();
                cx.subscribe(&editor, move |_this, _editor, _ev: &EditorEvent, cx| {
                    if let Some(this) = weak.upgrade() {
                        cx.update_entity(&this, |app, cx| app.on_editor_changed(cx));
                    }
                })
                .detach();
                let saved = content.clone();
                let doc = Document {
                    path: Some(path.clone()),
                    content,
                    saved_content: saved,
                    editor,
                    dirty: false,
                };
                self.docs.push(doc);
                self.active = self.docs.len() - 1;
                self.push_recent(path.clone());
                self.status = format!("Opened {}", path.display());
                self.persist();
            } else {
                self.status = "Open failed".to_string();
            }
        }
        cx.notify();
    }

    fn new_document(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        let dark = self.theme == Theme::Dark;
        let content = DEFAULT_TEMPLATE.to_string();
        let editor = cx.new(|cx| Editor::new(cx, content.clone(), dark));
        let weak = cx.weak_entity();
        cx.subscribe(&editor, move |_this, _editor, _ev: &EditorEvent, cx| {
            if let Some(this) = weak.upgrade() {
                cx.update_entity(&this, |app, cx| app.on_editor_changed(cx));
            }
        })
        .detach();
        let doc = Document {
            path: None,
            content: content.clone(),
            saved_content: content.clone(),
            editor,
            dirty: false,
        };
        self.docs.push(doc);
        self.active = self.docs.len() - 1;
        cx.notify();
    }

    fn close_tab_action(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        self.close_document(self.active, cx);
    }

    fn close_document(&mut self, idx: usize, cx: &mut Context<Self>) {
        if idx >= self.docs.len() {
            return;
        }
        if idx == self.active {
            self.sync_active(cx);
        }
        if !self.confirm_discard(&self.docs[idx]) {
            return;
        }
        self.docs.remove(idx);
        if self.docs.is_empty() {
            self.new_document_no_window(cx);
        }
        if self.active >= self.docs.len() {
            self.active = self.docs.len().saturating_sub(1);
        } else if idx < self.active {
            self.active -= 1;
        }
        cx.notify();
    }

    fn new_document_no_window(&mut self, cx: &mut Context<Self>) {
        let dark = self.theme == Theme::Dark;
        let content = DEFAULT_TEMPLATE.to_string();
        let editor = cx.new(|cx| Editor::new(cx, content.clone(), dark));
        let weak = cx.weak_entity();
        cx.subscribe(&editor, move |_this, _editor, _ev: &EditorEvent, cx| {
            if let Some(this) = weak.upgrade() {
                cx.update_entity(&this, |app, cx| app.on_editor_changed(cx));
            }
        })
        .detach();
        let doc = Document {
            path: None,
            content: content.clone(),
            saved_content: content.clone(),
            editor,
            dirty: false,
        };
        self.docs.push(doc);
        self.active = self.docs.len() - 1;
    }

    fn confirm_discard(&self, doc: &Document) -> bool {
        if !doc.dirty {
            return true;
        }
        let res = rfd::MessageDialog::new()
            .set_title("Unsaved changes")
            .set_description(format!(
                "Discard unsaved changes in {}?",
                doc.path
                    .as_ref()
                    .map(|p| p.display().to_string())
                    .unwrap_or_else(|| "Untitled".to_string())
            ))
            .set_buttons(rfd::MessageButtons::YesNo)
            .show();
        matches!(res, rfd::MessageDialogResult::Yes)
    }

    fn switch_tab(&mut self, idx: usize, cx: &mut Context<Self>) {
        if idx == self.active || idx >= self.docs.len() {
            return;
        }
        self.sync_active(cx);
        self.active = idx;
        cx.notify();
    }

    fn toggle_theme(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        self.theme = match self.theme {
            Theme::Dark => Theme::Light,
            Theme::Light => Theme::Dark,
        };
        let dark = self.theme == Theme::Dark;
        for doc in &self.docs {
            doc.editor.update(cx, |e, cx| e.set_dark(dark, cx));
        }
        self.persist();
        cx.notify();
    }

    fn set_view(&mut self, mode: ViewMode, _window: &mut Window, cx: &mut Context<Self>) {
        self.view_mode = mode;
        self.persist();
        cx.notify();
    }

    fn toggle_palette(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        self.palette_open = !self.palette_open;
        if self.palette_open {
            self.palette_query.clear();
            self.palette_selected = 0;
            // focus root to receive keys
            self.focus_handle.focus(_window);
        } else {
            // focus active editor
            if let Some(doc) = self.docs.get(self.active) {
                let fh = doc.editor.read(cx).focus_handle(cx).clone();
                fh.focus(_window);
            }
        }
        cx.notify();
    }

    fn palette_items(&self) -> Vec<String> {
        let mut items = vec![
            "New Document".to_string(),
            "Open...".to_string(),
            "Save".to_string(),
            "Compile".to_string(),
            "Toggle Theme".to_string(),
            "View: Split".to_string(),
            "View: Editor".to_string(),
            "View: Preview".to_string(),
            "Toggle Auto-compile".to_string(),
            "Toggle Log Panel".to_string(),
            "Close Tab".to_string(),
        ];
        for f in &self.recent_files {
            items.push(format!("Open Recent: {}", f));
        }
        if !self.palette_query.is_empty() {
            let q = self.palette_query.to_lowercase();
            items.retain(|i| i.to_lowercase().contains(&q));
        }
        items
    }

    fn palette_execute(&mut self, cmd: &str, window: &mut Window, cx: &mut Context<Self>) {
        match cmd {
            "New Document" => self.new_document(window, cx),
            "Open..." => self.open(window, cx),
            "Save" => self.save(window, cx),
            "Compile" => self.compile_current(cx),
            "Toggle Theme" => self.toggle_theme(window, cx),
            "View: Split" => self.set_view(ViewMode::Split, window, cx),
            "View: Editor" => self.set_view(ViewMode::Editor, window, cx),
            "View: Preview" => self.set_view(ViewMode::Preview, window, cx),
            "Toggle Auto-compile" => {
                self.auto_compile = !self.auto_compile;
                self.persist();
                self.show_toast(
                    ToastKind::Info,
                    format!("Auto-compile: {}", self.auto_compile),
                    cx,
                );
                cx.notify();
            }
            "Toggle Log Panel" => {
                self.log_open = !self.log_open;
                self.persist();
                cx.notify();
            }
            "Close Tab" => self.close_tab_action(window, cx),
            _ if cmd.starts_with("Open Recent: ") => {
                let path_str = cmd.trim_start_matches("Open Recent: ").to_string();
                let path = PathBuf::from(&path_str);
                if let Ok(content) = fs::read_to_string(&path) {
                    let dark = self.theme == Theme::Dark;
                    let editor = cx.new(|cx| Editor::new(cx, content.clone(), dark));
                    let weak = cx.weak_entity();
                    cx.subscribe(&editor, move |_this, _editor, _ev: &EditorEvent, cx| {
                        if let Some(this) = weak.upgrade() {
                            cx.update_entity(&this, |app, cx| app.on_editor_changed(cx));
                        }
                    })
                    .detach();
                    let saved = content.clone();
                    let doc = Document {
                        path: Some(path.clone()),
                        content,
                        saved_content: saved,
                        editor,
                        dirty: false,
                    };
                    self.docs.push(doc);
                    self.active = self.docs.len() - 1;
                    self.push_recent(path);
                    self.persist();
                    cx.notify();
                }
            }
            _ => {}
        }
        self.palette_open = false;
        // focus editor
        if let Some(doc) = self.docs.get(self.active) {
            let fh = doc.editor.read(cx).focus_handle(cx).clone();
            fh.focus(window);
        }
        cx.notify();
    }

    fn push_recent(&mut self, path: PathBuf) {
        let s = path.to_string_lossy().to_string();
        self.recent_files.retain(|p| p != &s);
        self.recent_files.insert(0, s);
        if self.recent_files.len() > 10 {
            self.recent_files.truncate(10);
        }
    }

    fn show_toast(&mut self, kind: ToastKind, message: String, cx: &mut Context<Self>) {
        self.toast_counter += 1;
        let id = self.toast_counter;
        self.toasts.push(Toast { id, kind, message });
        let this = cx.weak_entity();
        gpui::App::spawn(&*cx, async move |cx| {
            Timer::after(Duration::from_secs(3)).await;
            if let Some(e) = this.upgrade() {
                e.update(cx, |app, cx| {
                    app.toasts.retain(|t| t.id != id);
                    cx.notify();
                })
                .ok();
            }
        })
        .detach();
        cx.notify();
    }

    fn prev_page(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        if self.current_page > 1 {
            self.current_page -= 1;
            self.render_current_page();
            cx.notify();
        }
    }

    fn next_page(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        if self.page_count == 0 {
            return;
        }
        if self.current_page < self.page_count {
            self.current_page += 1;
            self.render_current_page();
            cx.notify();
        }
    }

    fn zoom_in(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        self.zoom = (self.zoom * 1.25).min(8.0);
        self.fit_width = false;
        self.render_current_page();
        cx.notify();
    }

    fn zoom_out(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        self.zoom = (self.zoom / 1.25).max(0.25);
        self.fit_width = false;
        self.render_current_page();
        cx.notify();
    }

    fn toggle_fit(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        self.fit_width = !self.fit_width;
        self.render_current_page();
        cx.notify();
    }

    fn check_updates(&mut self, cx: &mut Context<Self>) {
        self.updater_status = "Checking...".to_string();
        cx.notify();
        let this = cx.weak_entity();
        gpui::App::spawn(&*cx, async move |cx| {
            let status = cx
                .background_executor()
                .spawn(async move {
                    (|| -> Result<String, String> {
                        let resp = ureq::get(
                            "https://api.github.com/repos/shivamnarkar47/tikzforge/releases/latest",
                        )
                        .header("User-Agent", "tikzforge-gpui")
                        .call()
                        .map_err(|e| e.to_string())?;
                        let text = resp
                            .into_body()
                            .read_to_string()
                            .map_err(|e| e.to_string())?;
                        let json: serde_json::Value =
                            serde_json::from_str(&text).map_err(|e| e.to_string())?;
                        let tag = json
                            .get("tag_name")
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown");
                        if tag == "v0.2.1" || tag == "0.2.1" {
                            Ok("Up to date".to_string())
                        } else {
                            Ok(format!("Update: {}", tag))
                        }
                    })()
                    .unwrap_or_else(|_| "Update check failed".to_string())
                })
                .await;
            if let Some(e) = this.upgrade() {
                e.update(cx, |app, cx| {
                    app.updater_status = status;
                    cx.notify();
                })
                .ok();
            }
        })
        .detach();
    }
}

impl Focusable for TikzForgeApp {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for TikzForgeApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.window_bounds = Some(window.bounds());
        self.pdf_viewport_width = f32::from(window.bounds().size.width) * (1.0 - self.divider);

        let theme = self.theme;
        let bg = rgb(theme.bg());
        let fg = rgb(theme.fg());
        let panel = rgb(theme.panel());
        let accent = rgb(theme.accent());
        let muted = rgb(theme.muted());

        // sync active editor text (best effort)
        // (done on events)

        let title = self
            .active_doc()
            .path
            .as_ref()
            .map(|p| {
                p.file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string()
            })
            .unwrap_or_else(|| "Untitled".to_string());

        // tabs
        let tabs: Vec<AnyElement> = self
            .docs
            .iter()
            .enumerate()
            .map(|(i, doc)| {
                let active = i == self.active;
                let name = doc
                    .path
                    .as_ref()
                    .map(|p| {
                        p.file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string()
                    })
                    .unwrap_or_else(|| "Untitled".to_string());
                let label = if doc.dirty {
                    format!("{} •", name)
                } else {
                    name
                };
                let close_idx = i;
                div()
                    .id(SharedString::from(format!("tab-{}", i)))
                    .px_2()
                    .py_1()
                    .rounded_sm()
                    .cursor_pointer()
                    .when(active, |s| s.bg(accent).text_color(rgb(theme.bg())))
                    .when(!active, |s| s.hover(|h| h.bg(rgb(theme.bg()))))
                    .flex()
                    .items_center()
                    .gap_1()
                    .child(div().text_xs().child(label))
                    .child(
                        div()
                            .id(SharedString::from(format!("tab-close-{}", i)))
                            .text_xs()
                            .cursor_pointer()
                            .child("×")
                            .on_click(cx.listener(move |this, _, window, cx| {
                                this.close_document(close_idx, cx);
                                let _ = window;
                            })),
                    )
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.switch_tab(i, cx);
                        let _ = window;
                    }))
                    .into_any_element()
            })
            .collect();

        let menu_bar = div()
            .flex()
            .items_center()
            .gap_2()
            .px_3()
            .py_1()
            .bg(panel)
            .border_b_1()
            .border_color(rgb(theme.muted()))
            .child(
                div()
                    .text_sm()
                    .font_weight(FontWeight::BOLD)
                    .text_color(accent)
                    .child("TikzForge"),
            )
            .child(div().flex_1())
            .child(
                div()
                    .text_xs()
                    .text_color(muted)
                    .child(self.updater_status.clone()),
            )
            .child(
                div()
                    .id("open")
                    .px_2()
                    .py_1()
                    .rounded_sm()
                    .cursor_pointer()
                    .hover(|s| s.bg(rgb(theme.bg())))
                    .text_sm()
                    .child("Open")
                    .on_click(cx.listener(|this, _, window, cx| this.open(window, cx))),
            )
            .child(
                div()
                    .id("save")
                    .px_2()
                    .py_1()
                    .rounded_sm()
                    .cursor_pointer()
                    .hover(|s| s.bg(rgb(theme.bg())))
                    .text_sm()
                    .child("Save")
                    .on_click(cx.listener(|this, _, window, cx| this.save(window, cx))),
            )
            .child(
                div()
                    .id("compile")
                    .px_2()
                    .py_1()
                    .rounded_sm()
                    .cursor_pointer()
                    .hover(|s| s.bg(rgb(theme.bg())))
                    .text_sm()
                    .child("Compile")
                    .on_click(cx.listener(|this, _, _window, cx| this.compile_current(cx))),
            )
            .child(
                div()
                    .id("newdoc")
                    .px_2()
                    .py_1()
                    .rounded_sm()
                    .cursor_pointer()
                    .hover(|s| s.bg(rgb(theme.bg())))
                    .text_sm()
                    .child("New")
                    .on_click(cx.listener(|this, _, window, cx| this.new_document(window, cx))),
            )
            .child(
                div()
                    .id("theme")
                    .px_2()
                    .py_1()
                    .rounded_sm()
                    .cursor_pointer()
                    .hover(|s| s.bg(rgb(theme.bg())))
                    .text_sm()
                    .child(match theme {
                        Theme::Dark => "Light",
                        Theme::Light => "Dark",
                    })
                    .on_click(cx.listener(|this, _, window, cx| this.toggle_theme(window, cx))),
            )
            .child(
                div()
                    .id("palette")
                    .px_2()
                    .py_1()
                    .rounded_sm()
                    .cursor_pointer()
                    .hover(|s| s.bg(rgb(theme.bg())))
                    .text_sm()
                    .child("Palette")
                    .on_click(cx.listener(|this, _, window, cx| this.toggle_palette(window, cx))),
            );

        let tab_bar = div()
            .flex()
            .items_center()
            .gap_1()
            .px_2()
            .py_1()
            .bg(panel)
            .border_b_1()
            .border_color(rgb(theme.muted()))
            .children(tabs)
            .child(
                div()
                    .id("new-tab")
                    .px_2()
                    .py_1()
                    .rounded_sm()
                    .cursor_pointer()
                    .hover(|s| s.bg(rgb(theme.bg())))
                    .text_xs()
                    .child("+")
                    .on_click(cx.listener(|this, _, window, cx| this.new_document(window, cx))),
            );

        let view_tabs = div()
            .flex()
            .items_center()
            .gap_1()
            .px_3()
            .py_1()
            .bg(panel)
            .child(div().text_xs().text_color(muted).child("View:"))
            .children(
                [ViewMode::Split, ViewMode::Editor, ViewMode::Preview]
                    .iter()
                    .map(|&mode| {
                        let active = mode == self.view_mode;
                        div()
                            .id(SharedString::from(format!("view-{:?}", mode)))
                            .px_2()
                            .py_1()
                            .rounded_sm()
                            .cursor_pointer()
                            .text_xs()
                            .when(active, |s| s.bg(accent).text_color(rgb(theme.bg())))
                            .when(!active, |s| s.hover(|h| h.bg(rgb(theme.bg()))))
                            .child(mode.label())
                            .on_click(cx.listener(move |this, _, window, cx| {
                                this.set_view(mode, window, cx)
                            }))
                    }),
            );

        let editor_pane = if let Some(doc) = self.docs.get(self.active) {
            let editor_view = doc.editor.clone();
            div().flex_1().min_w(px(0.)).bg(panel).child(editor_view)
        } else {
            div().flex_1().child("No document")
        };

        let pdf_toolbar = div()
            .flex()
            .items_center()
            .gap_2()
            .px_2()
            .py_1()
            .bg(panel)
            .border_b_1()
            .border_color(rgb(theme.muted()))
            .child(
                div()
                    .id("pdf-prev")
                    .px_2()
                    .py_1()
                    .rounded_sm()
                    .cursor_pointer()
                    .hover(|s| s.bg(rgb(theme.bg())))
                    .text_xs()
                    .child("Prev")
                    .on_click(cx.listener(|this, _, window, cx| this.prev_page(window, cx))),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(fg)
                    .child(format!("{}/{}", self.current_page, self.page_count)),
            )
            .child(
                div()
                    .id("pdf-next")
                    .px_2()
                    .py_1()
                    .rounded_sm()
                    .cursor_pointer()
                    .hover(|s| s.bg(rgb(theme.bg())))
                    .text_xs()
                    .child("Next")
                    .on_click(cx.listener(|this, _, window, cx| this.next_page(window, cx))),
            )
            .child(div().flex_1())
            .child(
                div()
                    .id("pdf-zoom-out")
                    .px_2()
                    .py_1()
                    .rounded_sm()
                    .cursor_pointer()
                    .hover(|s| s.bg(rgb(theme.bg())))
                    .text_xs()
                    .child("-")
                    .on_click(cx.listener(|this, _, window, cx| this.zoom_out(window, cx))),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(fg)
                    .child(format!("{:.0}%", self.zoom * 100.0)),
            )
            .child(
                div()
                    .id("pdf-zoom-in")
                    .px_2()
                    .py_1()
                    .rounded_sm()
                    .cursor_pointer()
                    .hover(|s| s.bg(rgb(theme.bg())))
                    .text_xs()
                    .child("+")
                    .on_click(cx.listener(|this, _, window, cx| this.zoom_in(window, cx))),
            )
            .child(
                div()
                    .id("pdf-fit")
                    .px_2()
                    .py_1()
                    .rounded_sm()
                    .cursor_pointer()
                    .hover(|s| s.bg(rgb(theme.bg())))
                    .text_xs()
                    .child(if self.fit_width {
                        "Fit: On"
                    } else {
                        "Fit: Off"
                    })
                    .on_click(cx.listener(|this, _, window, cx| this.toggle_fit(window, cx))),
            );

        let pdf_content = if let Some(img_handle) = &self.pdf_image {
            let (w, h) = self.pdf_image_dims.unwrap_or((100, 100));
            let img_handle = img_handle.clone();
            div()
                .flex_1()
                .min_h(px(0.))
                .overflow_hidden()
                .flex()
                .items_start()
                .justify_center()
                .p_2()
                .child(
                    div()
                        .id("pdf-click")
                        .cursor_pointer()
                        .on_click(cx.listener(|this, event: &ClickEvent, window, cx| {
                            let pos = event.position();
                            let wb = window.bounds();
                            let origin_x = f32::from(wb.origin.x)
                                + f32::from(wb.size.width) * this.divider
                                + 12.0;
                            let origin_y = f32::from(wb.origin.y) + 70.0;
                            let rel_x = f32::from(pos.x) - origin_x;
                            let rel_y = f32::from(pos.y) - origin_y;
                            let scale = this.zoom.max(0.1);
                            let pdf_x = (rel_x / scale) as i32;
                            let pdf_y = (rel_y / scale) as i32;
                            if let Some(line) = find_source_position(
                                &this.synctex_records,
                                this.current_page as i32,
                                pdf_x,
                                pdf_y,
                            ) {
                                this.status = format!("SyncTeX → source line {}", line);
                            } else {
                                this.status =
                                    format!("SyncTeX: no record (click {},{})", pdf_x, pdf_y);
                            }
                            cx.notify();
                        }))
                        .child(img(img_handle).w(px(w as f32)).h(px(h as f32))),
                )
        } else {
            div().flex_1().flex().items_center().justify_center().child(
                div()
                    .text_xs()
                    .text_color(muted)
                    .child("No PDF. Compile to preview."),
            )
        };

        let pdf_pane = div()
            .flex_1()
            .min_w(px(0.))
            .flex()
            .flex_col()
            .bg(rgb(theme.bg()))
            .child(pdf_toolbar)
            .child(pdf_content);

        let divider = div()
            .w(px(6.))
            .h_full()
            .cursor_col_resize()
            .bg(rgb(theme.muted()))
            .hover(|s| s.bg(accent))
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _, _window, cx| {
                    this.resizing = true;
                    cx.notify();
                }),
            );

        let mut split = div()
            .flex()
            .flex_row()
            .flex_1()
            .min_h(px(0.))
            .on_mouse_move(cx.listener(|this, event: &MouseMoveEvent, window, cx| {
                if this.resizing {
                    let bounds = window.bounds();
                    let total = f32::from(bounds.size.width);
                    if total > 0.0 {
                        let x = f32::from(event.position.x) - f32::from(bounds.origin.x);
                        this.divider = (x / total).clamp(0.2, 0.8);
                        cx.notify();
                    }
                }
            }))
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(|this, _, _window, cx| {
                    if this.resizing {
                        this.resizing = false;
                        this.persist();
                        cx.notify();
                    }
                }),
            );

        match self.view_mode {
            ViewMode::Split => {
                split = split
                    .child(div().w(relative(self.divider)).h_full().child(editor_pane))
                    .child(divider)
                    .child(div().flex_1().h_full().child(pdf_pane));
            }
            ViewMode::Editor => {
                split = split.child(div().flex_1().h_full().child(editor_pane));
            }
            ViewMode::Preview => {
                split = split.child(div().flex_1().h_full().child(pdf_pane));
            }
        }

        // error list
        let error_list = if !self.errors.is_empty() {
            Some(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .px_2()
                    .py_1()
                    .bg(rgb(theme.panel()))
                    .border_t_1()
                    .border_color(rgb(theme.muted()))
                    .children(self.errors.iter().map(|e| {
                        let line = e.line;
                        let msg = e.message.clone();
                        div()
                            .id(SharedString::from(format!("err-{}-{}", line, msg.len())))
                            .text_xs()
                            .text_color(rgb(theme.muted()))
                            .cursor_pointer()
                            .child(format!("L{}: {}", line, msg))
                            .on_click(cx.listener(move |this, _, window, cx| {
                                if let Some(doc) = this.docs.get(this.active) {
                                    let editor = doc.editor.clone();
                                    cx.update_entity(&editor, |ed, cx| ed.goto_line(line, cx));
                                }
                                let _ = window;
                            }))
                    })),
            )
        } else {
            None
        };

        // log panel
        let log_panel = if self.log_open {
            Some(
                div()
                    .flex()
                    .flex_col()
                    .h(px(120.))
                    .bg(rgb(theme.panel()))
                    .border_t_1()
                    .border_color(rgb(theme.muted()))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .px_2()
                            .py_1()
                            .child(div().text_xs().text_color(muted).child("Log"))
                            .child(
                                div()
                                    .id("log-close")
                                    .text_xs()
                                    .cursor_pointer()
                                    .child("Hide")
                                    .on_click(cx.listener(|this, _, _w, cx| {
                                        this.log_open = false;
                                        this.persist();
                                        cx.notify();
                                    })),
                            ),
                    )
                    .child(
                        div()
                            .id("log-scroll")
                            .flex_1()
                            .min_h(px(0.))
                            .overflow_scroll()
                            .px_2()
                            .text_xs()
                            .text_color(fg)
                            .child(if self.log_text.is_empty() {
                                "No log output.".to_string()
                            } else {
                                self.log_text.clone()
                            }),
                    ),
            )
        } else {
            None
        };

        let status_bar = div()
            .flex()
            .items_center()
            .justify_between()
            .px_3()
            .py_1()
            .bg(panel)
            .border_t_1()
            .border_color(rgb(theme.muted()))
            .text_xs()
            .text_color(muted)
            .child(format!("{} — {}", title, self.status))
            .child(
                div()
                    .flex()
                    .gap_2()
                    .child(
                        div()
                            .id("log-toggle")
                            .cursor_pointer()
                            .child(if self.log_open {
                                "Hide Log"
                            } else {
                                "Show Log"
                            })
                            .on_click(cx.listener(|this, _, _w, cx| {
                                this.log_open = !this.log_open;
                                this.persist();
                                cx.notify();
                            })),
                    )
                    .child(
                        div().child("Ctrl+S Save  Ctrl+O Open  Ctrl+Enter Compile  Ctrl+K Palette"),
                    ),
            );

        // toast stack
        let toasts = self.toasts.clone();
        let toast_stack = div()
            .absolute()
            .bottom(px(40.))
            .right(px(16.))
            .flex()
            .flex_col()
            .gap_2()
            .children(toasts.iter().map(|t| {
                let color = match t.kind {
                    ToastKind::Success => rgb(0x98BB6C),
                    ToastKind::Error => rgb(0xE46876),
                    ToastKind::Info => rgb(0x7E9CD8),
                };
                div()
                    .px_3()
                    .py_2()
                    .rounded_sm()
                    .bg(color)
                    .text_xs()
                    .text_color(rgb(0x1F1F28))
                    .child(t.message.clone())
            }));

        // engine error card
        let engine_card = if !self.engine_ok {
            Some(
                div()
                    .absolute()
                    .top(px(60.))
                    .left(px(16.))
                    .w(px(420.))
                    .p_3()
                    .rounded_sm()
                    .bg(rgb(0xE46876))
                    .text_xs()
                    .text_color(rgb(0x1F1F28))
                    .child(format!(
                        "LaTeX engine not found. Expected at {}. Install tectonic or stage engine.",
                        self.engine_path.display()
                    )),
            )
        } else {
            None
        };

        // command palette
        let palette = if self.palette_open {
            let items = self.palette_items();
            let selected = self.palette_selected.min(items.len().saturating_sub(1));
            Some(
                div()
                    .absolute()
                    .inset_0()
                    .bg(rgba(0x00000088))
                    .flex()
                    .items_start()
                    .justify_center()
                    .pt(px(80.))
                    .child(
                        div()
                            .w(px(420.))
                            .max_h(px(360.))
                            .bg(panel)
                            .rounded_sm()
                            .border_1()
                            .border_color(rgb(theme.muted()))
                            .flex()
                            .flex_col()
                            .child(
                                div()
                                    .px_3()
                                    .py_2()
                                    .border_b_1()
                                    .border_color(rgb(theme.muted()))
                                    .text_sm()
                                    .child(if self.palette_query.is_empty() {
                                        "Type to filter...".to_string()
                                    } else {
                                        self.palette_query.clone()
                                    }),
                            )
                            .child(
                                div()
                                    .id("palette-scroll")
                                    .flex_1()
                                    .min_h(px(0.))
                                    .overflow_scroll()
                                    .children(items.iter().enumerate().map(|(i, cmd)| {
                                        let cmd = cmd.clone();
                                        let is_sel = i == selected;
                                        div()
                                            .id(SharedString::from(format!("palette-item-{}", i)))
                                            .px_3()
                                            .py_1()
                                            .cursor_pointer()
                                            .when(is_sel, |s| {
                                                s.bg(accent).text_color(rgb(theme.bg()))
                                            })
                                            .when(!is_sel, |s| s.hover(|h| h.bg(rgb(theme.bg()))))
                                            .text_xs()
                                            .child(cmd.clone())
                                            .on_click(cx.listener(move |this, _, window, cx| {
                                                this.palette_execute(cmd.as_str(), window, cx);
                                            }))
                                    })),
                            ),
                    ),
            )
        } else {
            None
        };

        let mut root = div()
            .flex()
            .flex_col()
            .size_full()
            .bg(bg)
            .text_color(fg)
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, window, cx| {
                if this.palette_open {
                    let ks = &event.keystroke;
                    if ks.key == "escape" {
                        this.palette_open = false;
                        if let Some(doc) = this.docs.get(this.active) {
                            let fh = doc.editor.read(cx).focus_handle(cx).clone();
                            fh.focus(window);
                        }
                        cx.notify();
                        return;
                    }
                    if ks.key == "enter" {
                        let items = this.palette_items();
                        let sel = this.palette_selected.min(items.len().saturating_sub(1));
                        if let Some(cmd) = items.get(sel).cloned() {
                            this.palette_execute(cmd.as_str(), window, cx);
                        }
                        return;
                    }
                    if ks.key == "up" {
                        if this.palette_selected > 0 {
                            this.palette_selected -= 1;
                        }
                        cx.notify();
                        return;
                    }
                    if ks.key == "down" {
                        let n = this.palette_items().len();
                        if this.palette_selected + 1 < n {
                            this.palette_selected += 1;
                        }
                        cx.notify();
                        return;
                    }
                    if ks.key == "backspace" {
                        this.palette_query.pop();
                        this.palette_selected = 0;
                        cx.notify();
                        return;
                    }
                    if !ks.modifiers.control
                        && !ks.modifiers.alt
                        && let Some(ch) = &ks.key_char
                        && !ch.is_empty()
                    {
                        this.palette_query.push_str(ch);
                        this.palette_selected = 0;
                        cx.notify();
                    }
                }
            }))
            .on_action(cx.listener(|this, _: &Save, window, cx| this.save(window, cx)))
            .on_action(cx.listener(|this, _: &Open, window, cx| this.open(window, cx)))
            .on_action(
                cx.listener(|this, _: &ToggleTheme, window, cx| this.toggle_theme(window, cx)),
            )
            .on_action(cx.listener(|this, _: &Compile, _window, cx| this.compile_current(cx)))
            .on_action(cx.listener(|this, _: &ViewSplit, window, cx| {
                this.set_view(ViewMode::Split, window, cx)
            }))
            .on_action(cx.listener(|this, _: &ViewEditor, window, cx| {
                this.set_view(ViewMode::Editor, window, cx)
            }))
            .on_action(cx.listener(|this, _: &ViewPreview, window, cx| {
                this.set_view(ViewMode::Preview, window, cx)
            }))
            .on_action(
                cx.listener(|this, _: &CommandPalette, window, cx| this.toggle_palette(window, cx)),
            )
            .on_action(
                cx.listener(|this, _: &NewDocument, window, cx| this.new_document(window, cx)),
            )
            .on_action(
                cx.listener(|this, _: &CloseTab, window, cx| this.close_tab_action(window, cx)),
            )
            .child(menu_bar)
            .child(tab_bar)
            .child(view_tabs)
            .child(split);

        if let Some(el) = error_list {
            root = root.child(el);
        }
        root = root.child(status_bar);
        if let Some(el) = log_panel {
            root = root.child(el);
        }

        // overlays
        root = root.child(toast_stack);
        if let Some(card) = engine_card {
            root = root.child(card);
        }
        if let Some(p) = palette {
            root = root.child(p);
        }

        root
    }
}

fn quit_app(_action: &QuitApp, cx: &mut App) {
    cx.quit();
}

fn main() {
    Application::new().run(|cx: &mut App| {
        cx.on_action(quit_app);
        cx.set_menus(vec![Menu {
            name: "TikzForge".into(),
            items: vec![
                MenuItem::action("New Document", NewDocument),
                MenuItem::action("Open...", Open),
                MenuItem::action("Save", Save),
                MenuItem::separator(),
                MenuItem::action("Toggle Theme", ToggleTheme),
                MenuItem::separator(),
                MenuItem::action("Quit", QuitApp),
            ],
        }]);

        cx.bind_keys([
            KeyBinding::new("ctrl-s", Save, None),
            KeyBinding::new("ctrl-o", Open, None),
            KeyBinding::new("ctrl-enter", Compile, None),
            KeyBinding::new("ctrl-k", CommandPalette, None),
            KeyBinding::new("ctrl-n", NewDocument, None),
            KeyBinding::new("ctrl-w", CloseTab, None),
        ]);

        let persisted2 = load_persisted();
        let bounds = if let (Some(x), Some(y), Some(w), Some(h)) = (
            persisted2.window_x,
            persisted2.window_y,
            persisted2.window_w,
            persisted2.window_h,
        ) {
            Bounds {
                origin: point(px(x), px(y)),
                size: size(px(w), px(h)),
            }
        } else {
            Bounds::centered(None, size(px(1200.), px(800.)), cx)
        };

        let view_handle = cx.new(|cx| TikzForgeApp::new(cx, persisted2));

        // check updates
        view_handle.update(cx, |app, cx| app.check_updates(cx));

        let vh = view_handle.clone();
        cx.on_window_closed(move |cx| {
            vh.update(cx, |this, _cx| {
                this.persist();
            });
            if cx.windows().is_empty() {
                cx.quit();
            }
        })
        .detach();

        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: None,
                ..Default::default()
            },
            |window, cx| {
                let view = view_handle.clone();
                view.read(cx).focus_handle(cx).focus(window);
                view
            },
        )
        .unwrap();

        cx.activate(true);
    });
}

#[cfg(test)]
mod tests {
    use super::{Persisted, TikzForgeApp, ViewMode, find_source_position, parse_synctex};
    use gpui::AppContext;

    #[test]
    fn app_constructs() {
        let mut cx = gpui::TestAppContext::single();
        let persisted = Persisted::default();
        let app = cx.new(|cx| TikzForgeApp::new(cx, persisted));
        cx.update(|appcx| {
            let a = app.read(appcx);
            assert_eq!(a.docs.len(), 1);
            assert_eq!(a.view_mode, ViewMode::Split);
        });
    }

    #[test]
    fn parse_compile_result_errors() {
        let mut cx = gpui::TestAppContext::single();
        let persisted = Persisted::default();
        let app = cx.new(|cx| TikzForgeApp::new(cx, persisted));
        cx.update(|appcx| {
            app.update(appcx, |app, _cx| {
                app.log_text = "error at l.7 foo\nerror at l.12 bar".to_string();
                app.parse_errors();
                assert!(!app.errors.is_empty());
                assert_eq!(app.errors[0].line, 7);
            });
        });
    }

    #[test]
    fn synctex_parse_round_trip() {
        let content = "1 10 20 5 0\n2 30 40 6 0\n";
        let recs = parse_synctex(content);
        assert_eq!(recs.len(), 2);
        assert_eq!(find_source_position(&recs, 1, 10, 20), Some(5));
    }
}
