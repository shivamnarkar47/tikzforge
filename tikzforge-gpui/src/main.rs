use gpui::*;
use gpui::prelude::FluentBuilder;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

// ---------- Theme ----------

#[derive(Copy, Clone, PartialEq, Eq)]
enum Theme {
    Dark, // Kanagawa Wave
    Light, // Kanagawa Lotus
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
    window_bounds: Option<Bounds<Pixels>>,
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
            // Save As dialog
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
}

impl Focusable for TikzForgeApp {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for TikzForgeApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.window_bounds = Some(window.bounds());

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

        // Menu bar (in-app)
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
                    .on_click(cx.listener(|_this, _, _window, _cx| {})),
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

        // View mode tabs
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

        // Editor placeholder
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

        // PDF placeholder
        let pdf_pane = div()
            .flex_1()
            .min_w(px(0.))
            .bg(rgb(theme.bg()))
            .p_3()
            .overflow_hidden()
            .child(
                div()
                    .text_xs()
                    .text_color(muted)
                    .child("PDF Preview (Phase 4)"),
            );

        // Divider
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

        // Split container
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

        // Status bar
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
            .on_action(cx.listener(|this, _: &Compile, _window, cx| { this.status = "Compile (Phase 4)".to_string(); cx.notify(); }))
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

// ---------- Main ----------

fn quit_app(_action: &QuitApp, cx: &mut App) {
    cx.quit();
}

fn main() {
    Application::new().run(|cx: &mut App| {
        // Native menu bar
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

        // Keybindings
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

        // Save geometry on close
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
            |_, _cx| view_handle.clone(),
        )
        .unwrap();

        cx.activate(true);
    });
}
