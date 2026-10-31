use codebook_tree_sitter_latex as ts_latex;
use gpui::*;
use gpui_component::highlighter::{LanguageConfig, LanguageRegistry};
use gpui_component::input::{Input, InputEvent, InputState, Position};
use gpui_component::{Theme as ComponentTheme, ThemeMode};
use std::sync::Once;

#[derive(Clone, Debug)]
pub enum EditorEvent {
    Changed,
}

const LATEX_HIGHLIGHTS: &str = r#"
(line_comment) @comment
(block_comment) @comment
(command_name) @function
(begin) @keyword
(end) @keyword
(inline_formula) @string.special
(displayed_equation) @string.special
"{" @punctuation.bracket
"}" @punctuation.bracket
"#;

static REGISTER_LATEX: Once = Once::new();

fn register_latex_highlighter() {
    REGISTER_LATEX.call_once(|| {
        let config = LanguageConfig::new(
            "latex",
            ts_latex::LANGUAGE.into(),
            vec![],
            LATEX_HIGHLIGHTS,
            "",
            "",
        );
        LanguageRegistry::singleton().register("latex", &config);
    });
}

/// GPUI Component code editor with a LaTeX tree-sitter highlighter.
///
/// The inner `InputState` needs a `Window`, so it is created lazily on the
/// first render. The wrapper keeps a plain-text mirror for callers that need
/// the current document without access to the window.
pub struct Editor {
    input: Option<Entity<InputState>>,
    text: String,
    pending_text: Option<String>,
    pending_line: Option<usize>,
    focus_handle: FocusHandle,
}

impl Editor {
    pub fn new(cx: &mut Context<Self>, initial: String, _dark: bool) -> Self {
        register_latex_highlighter();
        Self {
            input: None,
            text: initial,
            pending_text: None,
            pending_line: None,
            focus_handle: cx.focus_handle(),
        }
    }

    pub fn text(&self) -> String {
        self.text.clone()
    }

    pub fn goto_line(&mut self, line: usize, cx: &mut Context<Self>) {
        self.pending_line = Some(line.saturating_sub(1));
        cx.notify();
    }

    #[allow(dead_code)]
    pub fn set_text(&mut self, text: String, cx: &mut Context<Self>) {
        self.text = text.clone();
        self.pending_text = Some(text);
        cx.emit(EditorEvent::Changed);
        cx.notify();
    }

    pub fn set_dark(&mut self, dark: bool, cx: &mut Context<Self>) {
        ComponentTheme::global_mut(cx).mode = if dark {
            ThemeMode::Dark
        } else {
            ThemeMode::Light
        };
        if let Some(input) = &self.input {
            input.update(cx, |_, cx| cx.notify());
        }
        cx.notify();
    }
}

impl EventEmitter<EditorEvent> for Editor {}

impl Focusable for Editor {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for Editor {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.input.is_none() {
            let initial = self
                .pending_text
                .take()
                .unwrap_or_else(|| self.text.clone());
            let input = cx.new(|cx| {
                InputState::new(window, cx)
                    .code_editor("latex")
                    .line_number(true)
                    .soft_wrap(false)
                    .default_value(initial)
            });

            cx.subscribe(&input, move |this, input, _: &InputEvent, cx| {
                this.text = input.read(cx).value().to_string();
                cx.emit(EditorEvent::Changed);
                cx.notify();
            })
            .detach();

            self.input = Some(input);
        }

        let input = self.input.clone().expect("editor input");
        if let Some(text) = self.pending_text.take() {
            input.update(cx, |state, cx| state.set_value(text, window, cx));
        }
        if let Some(line) = self.pending_line.take() {
            input.update(cx, |state, cx| {
                state.set_cursor_position(Position::new(line as u32, 0), window, cx)
            });
        }
        if self.focus_handle.is_focused(window) {
            input.read(cx).focus_handle(cx).focus(window);
        }

        div().size_full().child(
            Input::new(&input)
                .appearance(false)
                .bordered(false)
                .focus_bordered(false)
                .h_full(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{Editor, LATEX_HIGHLIGHTS};
    use codebook_tree_sitter_latex as ts_latex;
    use gpui::AppContext;

    #[test]
    fn editor_buffer_round_trips() {
        let mut cx = gpui::TestAppContext::single();
        let ed = cx.new(|cx| Editor::new(cx, "hello".to_string(), true));
        cx.update(|app| {
            assert_eq!(ed.read(app).text(), "hello");
        });

        cx.update(|app| {
            ed.update(app, |e, cx| e.set_text("replaced".to_string(), cx));
        });
        cx.update(|app| {
            assert_eq!(ed.read(app).text(), "replaced");
        });
    }

    #[test]
    fn latex_highlight_query_compiles() {
        tree_sitter::Query::new(&ts_latex::LANGUAGE.into(), LATEX_HIGHLIGHTS)
            .expect("LaTeX highlight query should compile");
    }
}
