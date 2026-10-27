use crate::commands::filter_commands;
use codebook_tree_sitter_latex as ts_latex;
use gpui::prelude::FluentBuilder;
use gpui::*;
use ropey::{LineType, Rope};
use std::collections::{HashMap, HashSet};
use tree_sitter::Parser;

#[derive(Clone, Debug)]
pub enum EditorEvent {
    Changed,
}

const LT: LineType = LineType::LF_CR;

#[derive(Clone)]
pub struct Diagnostic {
    pub line: usize,
    pub message: String,
}

pub struct Editor {
    pub rope: Rope,
    pub cursor: usize,
    pub anchor: Option<usize>,
    pub dark: bool,
    parser: Parser,
    tree: Option<tree_sitter::Tree>,
    fold_ranges: Vec<(usize, usize)>,
    folded: HashSet<usize>,
    pub search_open: bool,
    pub search_query: String,
    pub replace_text: String,
    search_focus_query: bool,
    pub autocomplete_open: bool,
    pub autocomplete_items: Vec<&'static str>,
    pub autocomplete_selected: usize,
    autocomplete_prefix_start: usize,
    pub diagnostics: Vec<Diagnostic>,
    focus_handle: FocusHandle,
    bracket_highlights: HashSet<usize>,
}

impl Editor {
    pub fn new(cx: &mut Context<Self>, initial: String, dark: bool) -> Self {
        let mut parser = Parser::new();
        let _ = parser.set_language(&ts_latex::LANGUAGE.into());
        let rope = Rope::from_str(&initial);
        let tree = parser.parse(initial.clone(), None);
        Self {
            rope,
            cursor: 0,
            anchor: None,
            dark,
            parser,
            tree,
            fold_ranges: Vec::new(),
            folded: HashSet::new(),
            search_open: false,
            search_query: String::new(),
            replace_text: String::new(),
            search_focus_query: true,
            autocomplete_open: false,
            autocomplete_items: Vec::new(),
            autocomplete_selected: 0,
            autocomplete_prefix_start: 0,
            diagnostics: Vec::new(),
            focus_handle: cx.focus_handle(),
            bracket_highlights: HashSet::new(),
        }
    }

    pub fn text(&self) -> String {
        self.rope.to_string()
    }

    pub fn goto_line(&mut self, line: usize, cx: &mut Context<Self>) {
        let line = line
            .saturating_sub(1)
            .min(self.num_lines().saturating_sub(1));
        self.cursor = self.line_start_char(line);
        self.anchor = None;
        cx.notify();
    }

    pub fn set_text(&mut self, text: String, cx: &mut Context<Self>) {
        self.rope = Rope::from_str(&text);
        self.cursor = 0;
        self.anchor = None;
        self.reparse();
        self.update_folds();
        cx.emit(EditorEvent::Changed);
        cx.notify();
    }

    pub fn set_dark(&mut self, dark: bool, cx: &mut Context<Self>) {
        self.dark = dark;
        cx.notify();
    }

    pub fn set_diagnostics(&mut self, diags: Vec<Diagnostic>, cx: &mut Context<Self>) {
        self.diagnostics = diags;
        cx.notify();
    }

    fn colors(&self) -> Colors {
        if self.dark {
            Colors {
                bg: 0x1F1F28,
                fg: 0xDCD7BA,
                panel: 0x2A2A37,
                accent: 0x7E9CD8,
                muted: 0x727169,
                green: 0x98BB6C,
                red: 0xE46876,
                purple: 0x957FB8,
                cursor: 0xDCD7BA,
                selection: 0x2D4F67,
                line_hl: 0x2A2A37,
            }
        } else {
            Colors {
                bg: 0xF2ECBC,
                fg: 0x545464,
                panel: 0xE9E3C8,
                accent: 0x4C6A8D,
                muted: 0x8A8980,
                green: 0x6F894E,
                red: 0xC0392B,
                purple: 0x624483,
                cursor: 0x545464,
                selection: 0xC7D7E0,
                line_hl: 0xE9E3C8,
            }
        }
    }

    fn reparse(&mut self) {
        let text = self.rope.to_string();
        self.tree = self.parser.parse(text, None);
    }

    fn num_lines(&self) -> usize {
        self.rope.len_lines(LT).max(1)
    }

    fn line_start_char(&self, line: usize) -> usize {
        let line = line.min(self.num_lines().saturating_sub(1));
        let byte = self.rope.line_to_byte_idx(line, LT);
        self.rope.byte_to_char_idx(byte)
    }

    fn cursor_line_col(&self) -> (usize, usize) {
        let byte = self.char_to_byte(self.cursor);
        let line = self.rope.byte_to_line_idx(byte.min(self.rope.len()), LT);
        let ls = self.rope.line_to_byte_idx(line, LT);
        let col = self.rope.byte_to_char_idx(byte) - self.rope.byte_to_char_idx(ls);
        (line, col)
    }

    fn char_to_byte(&self, ci: usize) -> usize {
        self.rope.char_to_byte_idx(ci.min(self.rope.len_chars()))
    }

    fn byte_to_char_str(s: &str, byte: usize) -> usize {
        s.char_indices().take_while(|(i, _)| *i < byte).count()
    }

    fn selection_range(&self) -> Option<(usize, usize)> {
        self.anchor.map(|a| {
            if a <= self.cursor {
                (a, self.cursor)
            } else {
                (self.cursor, a)
            }
        })
    }

    fn delete_selection(&mut self) {
        if let Some((s, e)) = self.selection_range() {
            let sb = self.rope.char_to_byte_idx(s);
            let eb = self.rope.char_to_byte_idx(e);
            self.rope.remove(sb..eb);
            self.cursor = s;
            self.anchor = None;
        }
    }

    pub fn insert_text(&mut self, s: &str, cx: &mut Context<Self>) {
        if self.selection_range().is_some() {
            self.delete_selection();
        }
        if s.len() == 1 {
            let ch = s.chars().next().unwrap();
            match ch {
                '(' => {
                    self.insert_raw("()", 1);
                    self.finish_edit(cx);
                    return;
                }
                '[' => {
                    self.insert_raw("[]", 1);
                    self.finish_edit(cx);
                    return;
                }
                '{' => {
                    self.insert_raw("{}", 1);
                    self.finish_edit(cx);
                    return;
                }
                '"' | '\'' => {
                    let pair = format!("{}{}", ch, ch);
                    self.insert_raw(&pair, 1);
                    self.finish_edit(cx);
                    return;
                }
                ')' | ']' | '}' => {
                    let next = self.char_at(self.cursor);
                    if next == Some(ch) {
                        self.cursor += 1;
                        self.update_autocomplete();
                        cx.notify();
                        return;
                    }
                }
                _ => {}
            }
        }
        self.insert_raw(s, s.chars().count());
        self.finish_edit(cx);
    }

    fn finish_edit(&mut self, cx: &mut Context<Self>) {
        self.update_autocomplete();
        self.reparse();
        self.update_folds();
        cx.emit(EditorEvent::Changed);
        cx.notify();
    }

    fn insert_raw(&mut self, s: &str, cursor_advance: usize) {
        let byte = self.char_to_byte(self.cursor);
        self.rope.insert(byte, s);
        self.cursor += cursor_advance;
    }

    fn char_at(&self, ci: usize) -> Option<char> {
        if ci < self.rope.len_chars() {
            Some(self.rope.char(self.rope.char_to_byte_idx(ci)))
        } else {
            None
        }
    }

    pub fn backspace(&mut self, cx: &mut Context<Self>) {
        if self.selection_range().is_some() {
            self.delete_selection();
        } else if self.cursor > 0 {
            let ci = self.cursor - 1;
            let sb = self.rope.char_to_byte_idx(ci);
            let eb = self.rope.char_to_byte_idx(self.cursor);
            self.rope.remove(sb..eb);
            self.cursor = ci;
        }
        self.finish_edit(cx);
    }

    pub fn delete_forward(&mut self, cx: &mut Context<Self>) {
        if self.selection_range().is_some() {
            self.delete_selection();
        } else if self.cursor < self.rope.len_chars() {
            let sb = self.rope.char_to_byte_idx(self.cursor);
            let eb = self.rope.char_to_byte_idx(self.cursor + 1);
            self.rope.remove(sb..eb);
        }
        self.finish_edit(cx);
    }

    pub fn insert_newline(&mut self, cx: &mut Context<Self>) {
        if self.selection_range().is_some() {
            self.delete_selection();
        }
        let (line, col) = self.cursor_line_col();
        let _ = col;
        let line_text = self.rope.line(line, LT).to_string();
        let trimmed = line_text.trim_end_matches('\n').trim_end_matches('\r');
        let indent: String = trimmed.chars().take_while(|c| c.is_whitespace()).collect();
        let extra = if trimmed.trim_end().ends_with('{')
            || trimmed.trim_end().ends_with('[')
            || trimmed.trim_end().ends_with('(')
        {
            "  "
        } else {
            ""
        };
        let insertion = format!("\n{}{}", indent, extra);
        let adv = insertion.chars().count();
        self.insert_raw(&insertion, adv);
        self.finish_edit(cx);
    }

    pub fn indent(&mut self, cx: &mut Context<Self>) {
        self.insert_text("  ", cx);
    }

    pub fn dedent(&mut self, cx: &mut Context<Self>) {
        let (line, col) = self.cursor_line_col();
        if col >= 2 {
            let ls = self.line_start_char(line);
            let line_str: String = self.rope.line(line, LT).to_string();
            if line_str.starts_with("  ") {
                let sb = self.rope.char_to_byte_idx(ls);
                let eb = self.rope.char_to_byte_idx(ls + 2);
                self.rope.remove(sb..eb);
                self.cursor = self.cursor.saturating_sub(2);
                cx.emit(EditorEvent::Changed);
                cx.notify();
            }
        }
    }

    pub fn move_left(&mut self, shift: bool, cx: &mut Context<Self>) {
        if !shift {
            if let Some((s, _)) = self.selection_range() {
                self.cursor = s;
                self.anchor = None;
                cx.notify();
                return;
            }
        } else if self.anchor.is_none() {
            self.anchor = Some(self.cursor);
        }
        if self.cursor > 0 {
            self.cursor -= 1;
        }
        if !shift {
            self.anchor = None;
        }
        self.update_bracket_highlights();
        cx.notify();
    }

    pub fn move_right(&mut self, shift: bool, cx: &mut Context<Self>) {
        if !shift {
            if let Some((_, e)) = self.selection_range() {
                self.cursor = e;
                self.anchor = None;
                cx.notify();
                return;
            }
        } else if self.anchor.is_none() {
            self.anchor = Some(self.cursor);
        }
        let n = self.rope.len_chars();
        if self.cursor < n {
            self.cursor += 1;
        }
        if !shift {
            self.anchor = None;
        }
        self.update_bracket_highlights();
        cx.notify();
    }

    pub fn move_up(&mut self, shift: bool, cx: &mut Context<Self>) {
        if shift && self.anchor.is_none() {
            self.anchor = Some(self.cursor);
        }
        let (line, col) = self.cursor_line_col();
        if line > 0 {
            let new_line = line - 1;
            let new_line_len = self.line_char_len(new_line);
            let new_col = col.min(new_line_len);
            self.cursor = self.line_start_char(new_line) + new_col;
        }
        if !shift {
            self.anchor = None;
        }
        self.update_bracket_highlights();
        cx.notify();
    }

    pub fn move_down(&mut self, shift: bool, cx: &mut Context<Self>) {
        if shift && self.anchor.is_none() {
            self.anchor = Some(self.cursor);
        }
        let (line, col) = self.cursor_line_col();
        let last = self.num_lines().saturating_sub(1);
        if line < last {
            let new_line = line + 1;
            let new_line_len = self.line_char_len(new_line);
            let new_col = col.min(new_line_len);
            self.cursor = self.line_start_char(new_line) + new_col;
        }
        if !shift {
            self.anchor = None;
        }
        self.update_bracket_highlights();
        cx.notify();
    }

    pub fn move_home(&mut self, shift: bool, cx: &mut Context<Self>) {
        if shift && self.anchor.is_none() {
            self.anchor = Some(self.cursor);
        }
        let (line, _) = self.cursor_line_col();
        self.cursor = self.line_start_char(line);
        if !shift {
            self.anchor = None;
        }
        cx.notify();
    }

    pub fn move_end(&mut self, shift: bool, cx: &mut Context<Self>) {
        if shift && self.anchor.is_none() {
            self.anchor = Some(self.cursor);
        }
        let (line, _) = self.cursor_line_col();
        self.cursor = self.line_start_char(line) + self.line_char_len(line);
        if !shift {
            self.anchor = None;
        }
        cx.notify();
    }

    pub fn select_all(&mut self, cx: &mut Context<Self>) {
        self.anchor = Some(0);
        self.cursor = self.rope.len_chars();
        cx.notify();
    }

    fn line_char_len(&self, line: usize) -> usize {
        let line = line.min(self.num_lines().saturating_sub(1));
        let s = self.rope.line(line, LT).to_string();
        let s = s.trim_end_matches('\n');
        let s = s.trim_end_matches('\r');
        s.chars().count()
    }

    fn current_prefix(&self) -> (usize, String) {
        let text: String = self.rope.to_string();
        let chars: Vec<char> = text.chars().collect();
        let mut start = self.cursor;
        while start > 0 {
            let c = chars[start - 1];
            if c.is_alphanumeric() || c == '\\' {
                if c == '\\' {
                    start -= 1;
                    break;
                }
                start -= 1;
            } else {
                break;
            }
        }
        if start < self.cursor && start < chars.len() && chars[start] == '\\' {
            let prefix: String = chars[start..self.cursor].iter().collect();
            return (start, prefix);
        }
        (self.cursor, String::new())
    }

    fn update_autocomplete(&mut self) {
        let (start, prefix) = self.current_prefix();
        if prefix.starts_with('\\') {
            let items = filter_commands(&prefix);
            if !items.is_empty() {
                self.autocomplete_open = true;
                self.autocomplete_items = items;
                self.autocomplete_selected = 0;
                self.autocomplete_prefix_start = start;
                return;
            }
        }
        self.autocomplete_open = false;
        self.autocomplete_items.clear();
    }

    pub fn autocomplete_accept(&mut self, cx: &mut Context<Self>) {
        if !self.autocomplete_open {
            return;
        }
        if let Some(item) = self
            .autocomplete_items
            .get(self.autocomplete_selected)
            .copied()
        {
            let start = self.autocomplete_prefix_start;
            let end = self.cursor;
            let sb = self.rope.char_to_byte_idx(start);
            let eb = self.rope.char_to_byte_idx(end);
            self.rope.remove(sb..eb);
            let adv = item.chars().count();
            let byte = self.rope.char_to_byte_idx(start);
            self.rope.insert(byte, item);
            self.cursor = start + adv;
            self.anchor = None;
        }
        self.autocomplete_open = false;
        self.autocomplete_items.clear();
        self.update_autocomplete();
        self.reparse();
        self.update_folds();
        cx.emit(EditorEvent::Changed);
        cx.notify();
    }

    pub fn autocomplete_up(&mut self) {
        if self.autocomplete_open && self.autocomplete_selected > 0 {
            self.autocomplete_selected -= 1;
        }
    }
    pub fn autocomplete_down(&mut self) {
        if self.autocomplete_open && self.autocomplete_selected + 1 < self.autocomplete_items.len()
        {
            self.autocomplete_selected += 1;
        }
    }

    pub fn search_next(&mut self, cx: &mut Context<Self>) {
        if self.search_query.is_empty() {
            return;
        }
        let text = self.rope.to_string();
        let from = self.char_to_byte(self.cursor);
        if let Some(pos) = text[from..].find(&self.search_query) {
            let byte_pos = from + pos;
            let ci = self.rope.byte_to_char_idx(byte_pos);
            self.cursor = ci;
            self.anchor = Some(ci + self.search_query.chars().count());
            cx.notify();
        } else if let Some(pos) = text.find(&self.search_query) {
            let ci = self.rope.byte_to_char_idx(pos);
            self.cursor = ci;
            self.anchor = Some(ci + self.search_query.chars().count());
            cx.notify();
        }
    }

    pub fn replace_current(&mut self, cx: &mut Context<Self>) {
        if let Some((s, e)) = self.selection_range() {
            let selected: String = self
                .rope
                .slice(self.rope.char_to_byte_idx(s)..self.rope.char_to_byte_idx(e))
                .to_string();
            if selected == self.search_query {
                let sb = self.rope.char_to_byte_idx(s);
                let eb = self.rope.char_to_byte_idx(e);
                self.rope.remove(sb..eb);
                let byte = self.rope.char_to_byte_idx(s);
                self.rope.insert(byte, &self.replace_text);
                self.cursor = s + self.replace_text.chars().count();
                self.anchor = None;
            }
        }
        cx.emit(EditorEvent::Changed);
        self.search_next(cx);
    }

    pub fn replace_all(&mut self, cx: &mut Context<Self>) {
        let text = self.rope.to_string();
        let new = text.replace(&self.search_query, &self.replace_text);
        self.rope = Rope::from_str(&new);
        self.cursor = 0;
        self.anchor = None;
        self.reparse();
        self.update_folds();
        cx.emit(EditorEvent::Changed);
        cx.notify();
    }

    fn update_folds(&mut self) {
        let text = self.rope.to_string();
        let lines: Vec<&str> = text.lines().collect();
        let mut ranges = Vec::new();
        let mut stack: Vec<usize> = Vec::new();
        for (i, l) in lines.iter().enumerate() {
            let t = l.trim();
            if t.starts_with("\\begin{") {
                stack.push(i);
            } else if t.starts_with("\\end{") {
                if let Some(start) = stack.pop() {
                    if i > start + 1 {
                        ranges.push((start, i));
                    }
                }
            }
        }
        self.fold_ranges = ranges;
    }

    fn is_folded_hidden(&self, line: usize) -> bool {
        for (s, e) in &self.fold_ranges {
            if self.folded.contains(s) && line > *s && line <= *e {
                return true;
            }
        }
        false
    }

    pub fn toggle_fold(&mut self, start_line: usize, cx: &mut Context<Self>) {
        if self.folded.contains(&start_line) {
            self.folded.remove(&start_line);
        } else {
            self.folded.insert(start_line);
        }
        cx.notify();
    }

    fn is_fold_start(&self, line: usize) -> bool {
        self.fold_ranges.iter().any(|(s, _)| *s == line)
    }

    fn update_bracket_highlights(&mut self) {
        self.bracket_highlights.clear();
        let text: String = self.rope.to_string();
        let chars: Vec<char> = text.chars().collect();
        let n = chars.len();
        for &ci in &[self.cursor, self.cursor.saturating_sub(1)] {
            if ci >= n {
                continue;
            }
            let ch = chars[ci];
            let (open, close) = match ch {
                '(' => ('(', ')'),
                '[' => ('[', ']'),
                '{' => ('{', '}'),
                ')' => ('(', ')'),
                ']' => ('[', ']'),
                '}' => ('{', '}'),
                _ => continue,
            };
            if ch == open {
                let mut depth = 0i32;
                let mut j = ci;
                while j < n {
                    if chars[j] == open {
                        depth += 1;
                    } else if chars[j] == close {
                        depth -= 1;
                        if depth == 0 {
                            self.bracket_highlights.insert(ci);
                            self.bracket_highlights.insert(j);
                            break;
                        }
                    }
                    j += 1;
                }
            } else {
                let mut depth = 0i32;
                let mut j = ci as isize;
                while j >= 0 {
                    if chars[j as usize] == close {
                        depth += 1;
                    } else if chars[j as usize] == open {
                        depth -= 1;
                        if depth == 0 {
                            self.bracket_highlights.insert(ci);
                            self.bracket_highlights.insert(j as usize);
                            break;
                        }
                    }
                    j -= 1;
                }
            }
        }
    }

    pub fn handle_key(
        &mut self,
        event: &KeyDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let ks = &event.keystroke;
        let ctrl = ks.modifiers.control || ks.modifiers.platform;
        let shift = ks.modifiers.shift;
        let alt = ks.modifiers.alt;

        if self.search_open && self.search_focus_query {
            if ks.key == "tab" && !shift {
                self.search_focus_query = false;
                cx.notify();
                return;
            }
            if ks.key == "escape" {
                self.search_open = false;
                cx.notify();
                return;
            }
            if ks.key == "enter" {
                self.search_next(cx);
                return;
            }
            if !ctrl && !alt {
                if ks.key == "backspace" {
                    self.search_query.pop();
                    cx.notify();
                    return;
                }
                if let Some(ch) = &ks.key_char {
                    if ch.chars().count() == 1 {
                        self.search_query.push_str(ch);
                        cx.notify();
                        return;
                    }
                }
            }
        }
        if self.search_open && !self.search_focus_query {
            if ks.key == "tab" && shift {
                self.search_focus_query = true;
                cx.notify();
                return;
            }
            if ks.key == "escape" {
                self.search_open = false;
                cx.notify();
                return;
            }
            if !ctrl && !alt {
                if ks.key == "backspace" {
                    self.replace_text.pop();
                    cx.notify();
                    return;
                }
                if let Some(ch) = &ks.key_char {
                    if ch.chars().count() == 1 {
                        self.replace_text.push_str(ch);
                        cx.notify();
                        return;
                    }
                }
            }
        }

        if self.autocomplete_open {
            match ks.key.as_str() {
                "up" => {
                    self.autocomplete_up();
                    cx.notify();
                    return;
                }
                "down" => {
                    self.autocomplete_down();
                    cx.notify();
                    return;
                }
                "enter" | "tab" => {
                    self.autocomplete_accept(cx);
                    return;
                }
                "escape" => {
                    self.autocomplete_open = false;
                    cx.notify();
                    return;
                }
                _ => {}
            }
        }

        if ctrl {
            match ks.key.as_str() {
                "f" => {
                    self.search_open = true;
                    self.search_focus_query = true;
                    cx.notify();
                    return;
                }
                "a" => {
                    self.select_all(cx);
                    return;
                }
                _ => {}
            }
        }

        match ks.key.as_str() {
            "backspace" => {
                self.backspace(cx);
                return;
            }
            "delete" => {
                self.delete_forward(cx);
                return;
            }
            "enter" => {
                self.insert_newline(cx);
                return;
            }
            "tab" => {
                if shift {
                    self.dedent(cx);
                } else {
                    self.indent(cx);
                }
                return;
            }
            "left" => {
                self.move_left(shift, cx);
                return;
            }
            "right" => {
                self.move_right(shift, cx);
                return;
            }
            "up" => {
                self.move_up(shift, cx);
                return;
            }
            "down" => {
                self.move_down(shift, cx);
                return;
            }
            "home" => {
                self.move_home(shift, cx);
                return;
            }
            "end" => {
                self.move_end(shift, cx);
                return;
            }
            "escape" => {
                if self.search_open {
                    self.search_open = false;
                }
                self.anchor = None;
                cx.notify();
                return;
            }
            _ => {}
        }

        if !ctrl && !alt {
            if let Some(ch) = &ks.key_char {
                if !ch.is_empty() {
                    self.insert_text(ch, cx);
                    return;
                }
            }
            if ks.key == "space" {
                self.insert_text(" ", cx);
                return;
            }
        }
    }
}

pub struct Colors {
    pub bg: u32,
    pub fg: u32,
    pub panel: u32,
    pub accent: u32,
    pub muted: u32,
    pub green: u32,
    pub red: u32,
    pub purple: u32,
    pub cursor: u32,
    pub selection: u32,
    pub line_hl: u32,
}

impl EventEmitter<EditorEvent> for Editor {}

impl Focusable for Editor {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for Editor {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.update_bracket_highlights();
        if self.fold_ranges.is_empty() {
            self.update_folds();
        }
        let colors = self.colors();
        let num_lines = self.num_lines();
        let diag_lines: HashSet<usize> = self.diagnostics.iter().map(|d| d.line).collect();
        let bracket_chars = self.bracket_highlights.clone();
        let mut rows: Vec<AnyElement> = Vec::new();

        for line_idx in 0..num_lines {
            if self.is_folded_hidden(line_idx) {
                continue;
            }
            let line_str_raw = self
                .rope
                .line(line_idx.min(num_lines.saturating_sub(1)), LT)
                .to_string();
            let line_str = line_str_raw
                .trim_end_matches('\n')
                .trim_end_matches('\r')
                .to_string();
            let (cur_line, cur_col) = self.cursor_line_col();
            let is_cursor_line = cur_line == line_idx;
            let line_start = self.line_start_char(line_idx);
            let line_len = line_str.chars().count();
            let sel = self.selection_range();
            let foldable = self.is_fold_start(line_idx);
            let folded = self.folded.contains(&line_idx);
            let ln_color = if diag_lines.contains(&line_idx) {
                colors.red
            } else {
                colors.muted
            };

            let line_start_byte = self.rope.char_to_byte_idx(line_start);
            let line_end_byte = line_start_byte + line_str.len();
            let mut spans: Vec<(usize, usize, u32)> = Vec::new();
            if let Some(tree) = &self.tree {
                fn walk(node: tree_sitter::Node, spans: &mut Vec<(usize, usize, u32)>, dark: bool) {
                    if node.child_count() == 0 {
                        let kind = node.kind();
                        if let Some(c) = color_for_kind(kind, dark) {
                            spans.push((node.start_byte(), node.end_byte(), c));
                        }
                        return;
                    }
                    let mut c = node.walk();
                    for child in node.children(&mut c) {
                        walk(child, spans, dark);
                    }
                }
                walk(tree.root_node(), &mut spans, self.dark);
            }

            let mut line_spans: Vec<(usize, usize, u32)> = Vec::new();
            for (sb, eb, color) in &spans {
                if *eb <= line_start_byte || *sb >= line_end_byte {
                    continue;
                }
                let cs = (*sb).max(line_start_byte) - line_start_byte;
                let ce = (*eb).min(line_end_byte) - line_start_byte;
                let ci_start = Self::byte_to_char_str(&line_str, cs.min(line_str.len()));
                let ci_end = Self::byte_to_char_str(&line_str, ce.min(line_str.len()));
                if ci_end > ci_start {
                    line_spans.push((ci_start, ci_end, *color));
                }
            }
            line_spans.sort_by_key(|s| s.0);

            let (sel_start_char, sel_end_char) = if let Some((s, e)) = sel {
                (
                    s.saturating_sub(line_start).min(line_len),
                    e.saturating_sub(line_start).min(line_len),
                )
            } else {
                (0, 0)
            };
            let has_sel = sel.is_some() && sel_start_char < sel_end_char;
            let cursor_char_in_line = if is_cursor_line {
                cur_col.min(line_len)
            } else {
                0
            };

            let mut boundaries: Vec<usize> = vec![0, line_len];
            for (cs, ce, _) in &line_spans {
                boundaries.push(*cs);
                boundaries.push(*ce);
            }
            if has_sel {
                boundaries.push(sel_start_char);
                boundaries.push(sel_end_char);
            }
            if is_cursor_line {
                boundaries.push(cursor_char_in_line);
            }
            boundaries.sort();
            boundaries.dedup();

            let color_at = |ci: usize| -> u32 {
                for (cs, ce, color) in &line_spans {
                    if ci >= *cs && ci < *ce {
                        return *color;
                    }
                }
                colors.fg
            };

            let mut segments: Vec<AnyElement> = Vec::new();
            for w in boundaries.windows(2) {
                let a = w[0];
                let b = w[1];
                if a >= b {
                    continue;
                }
                let end = b.min(line_len);
                if a >= end {
                    continue;
                }
                let chunk: String = line_str.chars().skip(a).take(end - a).collect();
                if chunk.is_empty() {
                    continue;
                }
                let mut col = color_at(a);
                let global_idx = line_start + a;
                if bracket_chars.contains(&global_idx) {
                    col = colors.accent;
                }
                let is_selected = has_sel && a >= sel_start_char && b <= sel_end_char;
                let mut el = div().text_color(rgb(col)).child(chunk);
                if is_selected {
                    el = el.bg(rgb(colors.selection));
                }
                if bracket_chars.contains(&global_idx) {
                    el = el.font_weight(FontWeight::BOLD);
                }
                segments.push(el.into_any_element());
                if is_cursor_line && cursor_char_in_line == a {
                    segments.push(
                        div()
                            .w(px(2.))
                            .h(px(16.))
                            .bg(rgb(colors.cursor))
                            .into_any_element(),
                    );
                }
            }
            if is_cursor_line && cursor_char_in_line >= line_len {
                segments.push(
                    div()
                        .w(px(2.))
                        .h(px(16.))
                        .bg(rgb(colors.cursor))
                        .into_any_element(),
                );
            }

            let fold_indicator = if foldable {
                let start = line_idx;
                let label = if folded { "▸" } else { "▾" };
                div()
                    .id(SharedString::from(format!("fold-{}", start)))
                    .w(px(16.))
                    .text_color(rgb(colors.muted))
                    .cursor_pointer()
                    .child(label)
                    .on_click(cx.listener(move |this, _, _window, cx| {
                        this.toggle_fold(start, cx);
                    }))
            } else {
                div()
                    .id(SharedString::from(format!("fold-empty-{}", line_idx)))
                    .w(px(16.))
            };

            let row = div()
                .flex()
                .flex_row()
                .items_start()
                .min_h(px(18.))
                .child(fold_indicator)
                .child(
                    div()
                        .w(px(48.))
                        .text_color(rgb(ln_color))
                        .text_xs()
                        .text_right()
                        .pr_2()
                        .child(format!("{}", line_idx + 1)),
                )
                .child(
                    div()
                        .flex_1()
                        .flex()
                        .flex_row()
                        .items_center()
                        .min_h(px(18.))
                        .whitespace_nowrap()
                        .children(segments),
                );

            let row = if diag_lines.contains(&line_idx) {
                row.border_l_2().border_color(rgb(colors.red))
            } else {
                row
            };
            rows.push(row.into_any_element());
        }

        let autocomplete_el = if self.autocomplete_open && !self.autocomplete_items.is_empty() {
            let items: Vec<AnyElement> = self
                .autocomplete_items
                .iter()
                .enumerate()
                .map(|(i, cmd)| {
                    let selected = i == self.autocomplete_selected;
                    let cmd = *cmd;
                    div()
                        .px_2()
                        .py_1()
                        .when(selected, |s| s.bg(rgb(colors.selection)))
                        .text_color(rgb(colors.fg))
                        .text_sm()
                        .child(cmd)
                        .into_any_element()
                })
                .collect();
            Some(
                div()
                    .id("autocomplete")
                    .absolute()
                    .top(px(4.))
                    .right(px(4.))
                    .w(px(220.))
                    .max_h(px(240.))
                    .overflow_y_scroll()
                    .bg(rgb(colors.panel))
                    .border_1()
                    .border_color(rgb(colors.muted))
                    .rounded_sm()
                    .children(items),
            )
        } else {
            None
        };

        let search_panel = if self.search_open {
            Some(
                div()
                    .flex()
                    .flex_row()
                    .gap_2()
                    .px_2()
                    .py_1()
                    .bg(rgb(colors.panel))
                    .border_b_1()
                    .border_color(rgb(colors.muted))
                    .child(div().text_xs().text_color(rgb(colors.muted)).child("Find:"))
                    .child(
                        div()
                            .id("find-box")
                            .px_2()
                            .py_0p5()
                            .bg(rgb(colors.bg))
                            .text_sm()
                            .text_color(rgb(colors.fg))
                            .min_w(px(120.))
                            .child(if self.search_query.is_empty() {
                                " ".to_string()
                            } else {
                                self.search_query.clone()
                            }),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(colors.muted))
                            .child("Replace:"),
                    )
                    .child(
                        div()
                            .id("replace-box")
                            .px_2()
                            .py_0p5()
                            .bg(rgb(colors.bg))
                            .text_sm()
                            .text_color(rgb(colors.fg))
                            .min_w(px(120.))
                            .child(if self.replace_text.is_empty() {
                                " ".to_string()
                            } else {
                                self.replace_text.clone()
                            }),
                    )
                    .child(
                        div()
                            .id("search-next")
                            .px_2()
                            .py_0p5()
                            .rounded_sm()
                            .cursor_pointer()
                            .bg(rgb(colors.accent))
                            .text_xs()
                            .text_color(rgb(colors.bg))
                            .child("Next")
                            .on_click(cx.listener(|this, _, _w, cx| this.search_next(cx))),
                    )
                    .child(
                        div()
                            .id("search-replace")
                            .px_2()
                            .py_0p5()
                            .rounded_sm()
                            .cursor_pointer()
                            .bg(rgb(colors.accent))
                            .text_xs()
                            .text_color(rgb(colors.bg))
                            .child("Replace")
                            .on_click(cx.listener(|this, _, _w, cx| this.replace_current(cx))),
                    )
                    .child(
                        div()
                            .id("search-all")
                            .px_2()
                            .py_0p5()
                            .rounded_sm()
                            .cursor_pointer()
                            .bg(rgb(colors.accent))
                            .text_xs()
                            .text_color(rgb(colors.bg))
                            .child("All")
                            .on_click(cx.listener(|this, _, _w, cx| this.replace_all(cx))),
                    ),
            )
        } else {
            None
        };

        let mut container = div()
            .relative()
            .flex()
            .flex_col()
            .size_full()
            .bg(rgb(colors.bg))
            .text_color(rgb(colors.fg))
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, window, cx| {
                this.handle_key(event, window, cx);
            }));

        if let Some(sp) = search_panel {
            container = container.child(sp);
        }
        container = container.child(
            div()
                .id("editor-scroll")
                .flex_1()
                .min_h(px(0.))
                .overflow_y_scroll()
                .overflow_x_scroll()
                .children(rows),
        );
        if let Some(ac) = autocomplete_el {
            container = container.child(ac);
        }
        if let Some(first) = self.diagnostics.first() {
            container = container.child(
                div()
                    .px_2()
                    .py_1()
                    .bg(rgb(colors.red))
                    .text_xs()
                    .text_color(rgb(0xFFFFFF))
                    .child(first.message.clone()),
            );
        }
        container
    }
}

fn color_for_kind(kind: &str, dark: bool) -> Option<u32> {
    match kind {
        "command_name" | "todo_command_name" => Some(if dark { 0x7E9CD8 } else { 0x4C6A8D }),
        k if k.contains("comment") => Some(if dark { 0x98BB6C } else { 0x6F894E }),
        "$" | "$$" => Some(if dark { 0x7E9CD8 } else { 0x4C6A8D }),
        "(" | ")" | "[" | "]" | "{" | "}" => Some(if dark { 0x727169 } else { 0x8A8980 }),
        _ => None,
    }
}
