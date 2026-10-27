use gpui::*;
use gpui::prelude::FluentBuilder;
use pdfium_bundled::bind_pdfium_silent;
use pdfium_bundled::pdfium_render::prelude::Pdfium;
use serde::{Deserialize, Serialize};
use smallvec::smallvec;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

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
        if !trimmed.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false) {
            continue;
        }
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.len() >= 5 {
            if let (Ok(page), Ok(x), Ok(y), Ok(line)) = (
                parts[0].parse::<i32>(),
                parts[1].parse::<i32>(),
                parts[2].parse::<i32>(),
                parts[3].parse::<i32>(),
            ) {
                records.push(SynctexRecord {
                    page,
                    x,
                    y,
                    line,
                });
            }
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
    file_path: Option<String>,
}

fn config_path() -> PathBuf {
    let base = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    base.join("tikzforge").join("gpui.json")
}

fn load_persisted() -> Persisted {
    let path = config_path();
    if let Some(p) = fs::read_to_string(&path).ok().and_then(|s| serde_json::from_str(&s).ok()) {
        p
    } else {
        Persisted {
            dark: true,
            view_mode: "split".to_string(),
            divider: 0.5,
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
        QuitApp
    ]
);

// ---------- App state ----------

struct TikzForgeApp {
    theme: Theme,
    view_mode: ViewMode,
    divider: f32,
    resizing: bool,
    file_path: Option<PathBuf>,
    content: String,
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
}

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
        let file_path = persisted.file_path.map(PathBuf::from);
        let content = file_path
            .as_ref()
            .and_then(|p| fs::read_to_string(p).ok())
            .unwrap_or_else(|| {
                "\\documentclass{standalone}\n\\usepackage{tikz}\n\\begin{document}\n\\begin{tikzpicture}\n  \\draw (0,0) -- (1,1);\n\\end{tikzpicture}\n\\end{document}\n".to_string()
            });
        Self {
            theme,
            view_mode,
            divider,
            resizing: false,
            file_path,
            content,
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
        }
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
            file_path: self.file_path.as_ref().map(|p| p.to_string_lossy().to_string()),
        };
        save_persisted(&p);
    }

    fn save(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        if let Some(path) = &self.file_path.clone() {
            if fs::write(path, &self.content).is_ok() {
                self.status = format!("Saved {}", path.display());
            } else {
                self.status = "Save failed".to_string();
            }
        } else {
            let dialog = rfd::FileDialog::new()
                .add_filter("TeX", &["tex"])
                .set_file_name("untitled.tex");
            if let Some(path) = dialog.save_file() {
                if fs::write(&path, &self.content).is_ok() {
                    self.file_path = Some(path.clone());
                    self.status = format!("Saved {}", path.display());
                } else {
                    self.status = "Save failed".to_string();
                }
            }
        }
        self.persist();
        cx.notify();
    }

    fn open(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        let dialog = rfd::FileDialog::new().add_filter("TeX", &["tex"]);
        if let Some(path) = dialog.pick_file() {
            if let Ok(content) = fs::read_to_string(&path) {
                self.content = content;
                self.file_path = Some(path.clone());
                self.status = format!("Opened {}", path.display());
            } else {
                self.status = "Open failed".to_string();
            }
        }
        self.persist();
        cx.notify();
    }

    fn toggle_theme(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        self.theme = match self.theme {
            Theme::Dark => Theme::Light,
            Theme::Light => Theme::Dark,
        };
        self.persist();
        cx.notify();
    }

    fn set_view(&mut self, mode: ViewMode, _window: &mut Window, cx: &mut Context<Self>) {
        self.view_mode = mode;
        self.persist();
        cx.notify();
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

    fn compile(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        self.status = "Compiling...".to_string();
        cx.notify();
        let content = self.content.clone();
        let path = self
            .file_path
            .clone()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| "untitled.tex".to_string());
        match tikzforge_core::compile_tex(path, content, None, None) {
            Ok(cr) => {
                if cr.success {
                    if let Some(pdf) = cr.pdf {
                        self.pdf_bytes = Some(pdf);
                        self.current_page = 1;
                        self.render_current_page();
                        if let Some(p) = &self.file_path {
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
                    } else {
                        self.status = "Compile succeeded but no PDF".to_string();
                    }
                } else {
                    self.status = "Compile failed".to_string();
                }
            }
            Err(e) => {
                self.status = format!("Compile error: {}", e);
            }
        }
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

        let title = self
            .file_path
            .as_ref()
            .map(|p| p.file_name().unwrap_or_default().to_string_lossy().to_string())
            .unwrap_or_else(|| "Untitled".to_string());

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
                    .on_click(cx.listener(|this, _, window, cx| this.compile(window, cx))),
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
            );

        let view_tabs = div()
            .flex()
            .items_center()
            .gap_1()
            .px_3()
            .py_1()
            .bg(panel)
            .child(
                div()
                    .text_xs()
                    .text_color(muted)
                    .child("View:"),
            )
            .children([ViewMode::Split, ViewMode::Editor, ViewMode::Preview].iter().map(
                |&mode| {
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
                },
            ));

        let editor_pane = div()
            .flex_1()
            .min_w(px(0.))
            .bg(panel)
            .p_3()
            .overflow_hidden()
            .child(
                div()
                    .text_xs()
                    .text_color(muted)
                    .child("Editor (Phase 3)"),
            )
            .child(
                div()
                    .mt_2()
                    .text_sm()
                    .text_color(fg)
                    .child(self.content.clone()),
            );

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
                    .child(if self.fit_width { "Fit: On" } else { "Fit: Off" })
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
                                this.status = format!(
                                    "SyncTeX: no record (click {},{})",
                                    pdf_x, pdf_y
                                );
                            }
                            cx.notify();
                        }))
                        .child(img(img_handle).w(px(w as f32)).h(px(h as f32))),
                )
        } else {
            div()
                .flex_1()
                .flex()
                .items_center()
                .justify_center()
                .child(
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
            .child(div().child("Ctrl+S Save  Ctrl+O Open  Ctrl+Enter Compile  Ctrl+K Palette"));

        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(bg)
            .text_color(fg)
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(|this, _: &Save, window, cx| this.save(window, cx)))
            .on_action(cx.listener(|this, _: &Open, window, cx| this.open(window, cx)))
            .on_action(cx.listener(|this, _: &ToggleTheme, window, cx| this.toggle_theme(window, cx)))
            .on_action(cx.listener(|this, _: &Compile, window, cx| this.compile(window, cx)))
            .on_action(cx.listener(|this, _: &ViewSplit, window, cx| this.set_view(ViewMode::Split, window, cx)))
            .on_action(cx.listener(|this, _: &ViewEditor, window, cx| this.set_view(ViewMode::Editor, window, cx)))
            .on_action(cx.listener(|this, _: &ViewPreview, window, cx| this.set_view(ViewMode::Preview, window, cx)))
            .on_action(cx.listener(|this, _: &CommandPalette, _window, cx| {
                this.status = "Command palette (Phase 5)".to_string();
                cx.notify();
            }))
            .child(menu_bar)
            .child(view_tabs)
            .child(split)
            .child(status_bar)
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