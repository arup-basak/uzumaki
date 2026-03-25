use unicode_segmentation::UnicodeSegmentation;
use winit::keyboard::{Key, NamedKey};

// ── Selection ────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct Selection {
    /// Anchor point (where selection started), grapheme index
    pub anchor: usize,
    /// Active point / cursor position, grapheme index
    pub active: usize,
}

impl Selection {
    pub fn new() -> Self {
        Self {
            anchor: 0,
            active: 0,
        }
    }

    pub fn is_collapsed(&self) -> bool {
        self.anchor == self.active
    }

    pub fn start(&self) -> usize {
        self.anchor.min(self.active)
    }

    pub fn end(&self) -> usize {
        self.anchor.max(self.active)
    }

    pub fn set_cursor(&mut self, pos: usize) {
        self.anchor = pos;
        self.active = pos;
    }
}

// ── EditEvent ────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub enum EditKind {
    Insert,
    DeleteBackward,
    DeleteForward,
    DeleteWordBackward,
    DeleteWordForward,
}

#[derive(Clone, Debug)]
pub struct EditEvent {
    pub kind: EditKind,
    pub inserted: Option<String>,
}

// ── KeyResult ────────────────────────────────────────────────────────

pub enum KeyResult {
    Edit(EditEvent),
    Blur,
    Handled,
    Ignored,
}

// ── TextModel ────────────────────────────────────────────────────────

pub struct TextModel {
    // string for now :D
    pub text: String,
    pub selection: Selection,
    pub max_length: Option<usize>,
    pub multiline: bool,
}

impl TextModel {
    pub fn new() -> Self {
        Self {
            text: String::new(),
            selection: Selection::new(),
            max_length: None,
            multiline: false,
        }
    }

    pub fn grapheme_count(&self) -> usize {
        self.text.graphemes(true).count()
    }

    fn grapheme_to_byte(&self, idx: usize) -> usize {
        self.text
            .grapheme_indices(true)
            .nth(idx)
            .map(|(i, _)| i)
            .unwrap_or(self.text.len())
    }

    /// Returns the selected text slice, or empty string if selection is collapsed.
    pub fn selected_text(&self) -> &str {
        if self.selection.is_collapsed() {
            return "";
        }
        let start = self.grapheme_to_byte(self.selection.start());
        let end = self.grapheme_to_byte(self.selection.end());
        &self.text[start..end]
    }

    /// Delete selected text. Returns true if something was deleted.
    pub fn delete_selection(&mut self) -> bool {
        if self.selection.is_collapsed() {
            return false;
        }
        let start = self.selection.start();
        let end = self.selection.end();
        let byte_start = self.grapheme_to_byte(start);
        let byte_end = self.grapheme_to_byte(end);
        self.text.replace_range(byte_start..byte_end, "");
        self.selection.set_cursor(start);
        true
    }

    pub fn set_value(&mut self, value: String) {
        if self.text == value {
            return;
        }
        self.text = value;
        let count = self.grapheme_count();
        if self.selection.active > count {
            self.selection.active = count;
        }
        if self.selection.anchor > count {
            self.selection.anchor = count;
        }
    }

    /// Insert text at cursor. Newlines rejected when `!self.multiline`.
    pub fn insert(&mut self, ch: &str) -> Option<EditEvent> {
        // Single-line: reject newlines at the boundary
        let text_to_insert = if !self.multiline {
            let filtered: String = ch.chars().filter(|&c| c != '\n' && c != '\r').collect();
            if filtered.is_empty() {
                return None;
            }
            filtered
        } else {
            ch.to_string()
        };

        if let Some(max) = self.max_length {
            let current = self.grapheme_count() - (self.selection.end() - self.selection.start());
            let insert_count = text_to_insert.graphemes(true).count();
            if current + insert_count > max {
                return None;
            }
        }

        self.delete_selection();
        let byte_pos = self.grapheme_to_byte(self.selection.active);
        self.text.insert_str(byte_pos, &text_to_insert);
        let inserted = text_to_insert.graphemes(true).count();
        self.selection.active += inserted;
        self.selection.anchor = self.selection.active;
        Some(EditEvent {
            kind: EditKind::Insert,
            inserted: Some(text_to_insert),
        })
    }

    pub fn delete_backward(&mut self) -> Option<EditEvent> {
        if self.delete_selection() {
            return Some(EditEvent {
                kind: EditKind::DeleteBackward,
                inserted: None,
            });
        }
        if self.selection.active == 0 {
            return None;
        }
        let end_byte = self.grapheme_to_byte(self.selection.active);
        self.selection.active -= 1;
        self.selection.anchor = self.selection.active;
        let start_byte = self.grapheme_to_byte(self.selection.active);
        self.text.replace_range(start_byte..end_byte, "");
        Some(EditEvent {
            kind: EditKind::DeleteBackward,
            inserted: None,
        })
    }

    pub fn delete_forward(&mut self) -> Option<EditEvent> {
        if self.delete_selection() {
            return Some(EditEvent {
                kind: EditKind::DeleteForward,
                inserted: None,
            });
        }
        let count = self.grapheme_count();
        if self.selection.active >= count {
            return None;
        }
        let start_byte = self.grapheme_to_byte(self.selection.active);
        let end_byte = self.grapheme_to_byte(self.selection.active + 1);
        self.text.replace_range(start_byte..end_byte, "");
        Some(EditEvent {
            kind: EditKind::DeleteForward,
            inserted: None,
        })
    }

    pub fn delete_word_backward(&mut self) -> Option<EditEvent> {
        if self.delete_selection() {
            return Some(EditEvent {
                kind: EditKind::DeleteWordBackward,
                inserted: None,
            });
        }
        if self.selection.active == 0 {
            return None;
        }
        let end = self.selection.active;
        let graphemes: Vec<&str> = self.text.graphemes(true).collect();
        let mut pos = end;
        while pos > 0 && graphemes[pos - 1].chars().all(char::is_whitespace) {
            pos -= 1;
        }
        while pos > 0 && !graphemes[pos - 1].chars().all(char::is_whitespace) {
            pos -= 1;
        }
        let byte_start = self.grapheme_to_byte(pos);
        let byte_end = self.grapheme_to_byte(end);
        self.text.replace_range(byte_start..byte_end, "");
        self.selection.set_cursor(pos);
        Some(EditEvent {
            kind: EditKind::DeleteWordBackward,
            inserted: None,
        })
    }

    pub fn delete_word_forward(&mut self) -> Option<EditEvent> {
        if self.delete_selection() {
            return Some(EditEvent {
                kind: EditKind::DeleteWordForward,
                inserted: None,
            });
        }
        let count = self.grapheme_count();
        if self.selection.active >= count {
            return None;
        }
        let start = self.selection.active;
        let graphemes: Vec<&str> = self.text.graphemes(true).collect();
        let mut pos = start;
        while pos < count && !graphemes[pos].chars().all(char::is_whitespace) {
            pos += 1;
        }
        while pos < count && graphemes[pos].chars().all(char::is_whitespace) {
            pos += 1;
        }
        let byte_start = self.grapheme_to_byte(start);
        let byte_end = self.grapheme_to_byte(pos);
        self.text.replace_range(byte_start..byte_end, "");
        Some(EditEvent {
            kind: EditKind::DeleteWordForward,
            inserted: None,
        })
    }

    // ── Movement ─────────────────────────────────────────────────────

    pub fn move_left(&mut self, extend: bool) {
        if !extend && !self.selection.is_collapsed() {
            let pos = self.selection.start();
            self.selection.set_cursor(pos);
        } else if self.selection.active > 0 {
            self.selection.active -= 1;
            if !extend {
                self.selection.anchor = self.selection.active;
            }
        }
    }

    pub fn move_right(&mut self, extend: bool) {
        let count = self.grapheme_count();
        if !extend && !self.selection.is_collapsed() {
            let pos = self.selection.end();
            self.selection.set_cursor(pos);
        } else if self.selection.active < count {
            self.selection.active += 1;
            if !extend {
                self.selection.anchor = self.selection.active;
            }
        }
    }

    pub fn move_word_left(&mut self, extend: bool) {
        let graphemes: Vec<&str> = self.text.graphemes(true).collect();
        let mut pos = self.selection.active;
        while pos > 0 && graphemes[pos - 1].chars().all(char::is_whitespace) {
            pos -= 1;
        }
        while pos > 0 && !graphemes[pos - 1].chars().all(char::is_whitespace) {
            pos -= 1;
        }
        self.selection.active = pos;
        if !extend {
            self.selection.anchor = pos;
        }
    }

    pub fn move_word_right(&mut self, extend: bool) {
        let graphemes: Vec<&str> = self.text.graphemes(true).collect();
        let count = graphemes.len();
        let mut pos = self.selection.active;
        while pos < count && !graphemes[pos].chars().all(char::is_whitespace) {
            pos += 1;
        }
        while pos < count && graphemes[pos].chars().all(char::is_whitespace) {
            pos += 1;
        }
        self.selection.active = pos;
        if !extend {
            self.selection.anchor = pos;
        }
    }

    pub fn move_home(&mut self, extend: bool) {
        self.move_line_start(extend);
    }

    pub fn move_end(&mut self, extend: bool) {
        self.move_line_end(extend);
    }

    pub fn move_absolute_home(&mut self, extend: bool) {
        self.selection.active = 0;
        if !extend {
            self.selection.anchor = 0;
        }
    }

    pub fn move_absolute_end(&mut self, extend: bool) {
        let count = self.grapheme_count();
        self.selection.active = count;
        if !extend {
            self.selection.anchor = count;
        }
    }

    pub fn move_line_start(&mut self, extend: bool) {
        let graphemes: Vec<&str> = self.text.graphemes(true).collect();
        let mut pos = self.selection.active;
        while pos > 0 && graphemes[pos - 1] != "\n" {
            pos -= 1;
        }
        self.selection.active = pos;
        if !extend {
            self.selection.anchor = pos;
        }
    }

    pub fn move_line_end(&mut self, extend: bool) {
        let graphemes: Vec<&str> = self.text.graphemes(true).collect();
        let count = graphemes.len();
        let mut pos = self.selection.active;
        while pos < count && graphemes[pos] != "\n" {
            pos += 1;
        }
        self.selection.active = pos;
        if !extend {
            self.selection.anchor = pos;
        }
    }

    /// Move cursor to a specific grapheme index. Used by caller for vertical nav and mouse clicks.
    pub fn move_to(&mut self, pos: usize, extend: bool) {
        let count = self.grapheme_count();
        self.selection.active = pos.min(count);
        if !extend {
            self.selection.anchor = self.selection.active;
        }
    }

    pub fn select_all(&mut self) {
        self.selection.anchor = 0;
        self.selection.active = self.grapheme_count();
    }

    pub fn word_at(&self, grapheme_idx: usize) -> (usize, usize) {
        let graphemes: Vec<&str> = self.text.graphemes(true).collect();
        if graphemes.is_empty() {
            return (0, 0);
        }
        let idx = grapheme_idx.min(graphemes.len().saturating_sub(1));

        let mut start = idx;
        while start > 0 && !graphemes[start - 1].chars().all(char::is_whitespace) {
            start -= 1;
        }

        let mut end = idx;
        while end < graphemes.len() && !graphemes[end].chars().all(char::is_whitespace) {
            end += 1;
        }

        (start, end)
    }

    // ── Key handling ─────────────────────────────────────────────────

    /// Handle a key press. Returns `Ignored` for ArrowUp/ArrowDown —
    /// the caller must resolve vertical navigation externally via `move_to`.
    pub fn handle_key(&mut self, key: &Key, modifiers: u32) -> KeyResult {
        let shift = modifiers & 4 != 0;
        let ctrl = modifiers & 1 != 0;

        match key {
            Key::Character(ch) => {
                if ctrl {
                    if ch.eq_ignore_ascii_case("a") {
                        self.select_all();
                        return KeyResult::Handled;
                    }
                    return KeyResult::Ignored;
                }
                match self.insert(ch) {
                    Some(edit) => KeyResult::Edit(edit),
                    None => KeyResult::Handled,
                }
            }
            Key::Named(named) => match named {
                NamedKey::Backspace => {
                    if ctrl {
                        match self.delete_word_backward() {
                            Some(edit) => KeyResult::Edit(edit),
                            None => KeyResult::Handled,
                        }
                    } else {
                        match self.delete_backward() {
                            Some(edit) => KeyResult::Edit(edit),
                            None => KeyResult::Handled,
                        }
                    }
                }
                NamedKey::Delete => {
                    if ctrl {
                        match self.delete_word_forward() {
                            Some(edit) => KeyResult::Edit(edit),
                            None => KeyResult::Handled,
                        }
                    } else {
                        match self.delete_forward() {
                            Some(edit) => KeyResult::Edit(edit),
                            None => KeyResult::Handled,
                        }
                    }
                }
                NamedKey::ArrowLeft => {
                    if ctrl {
                        self.move_word_left(shift);
                    } else {
                        self.move_left(shift);
                    }
                    KeyResult::Handled
                }
                NamedKey::ArrowRight => {
                    if ctrl {
                        self.move_word_right(shift);
                    } else {
                        self.move_right(shift);
                    }
                    KeyResult::Handled
                }
                // ArrowUp/ArrowDown: return Ignored so caller handles vertical nav
                NamedKey::ArrowUp | NamedKey::ArrowDown => KeyResult::Ignored,
                NamedKey::Home => {
                    if ctrl {
                        self.move_absolute_home(shift);
                    } else {
                        self.move_home(shift);
                    }
                    KeyResult::Handled
                }
                NamedKey::End => {
                    if ctrl {
                        self.move_absolute_end(shift);
                    } else {
                        self.move_end(shift);
                    }
                    KeyResult::Handled
                }
                NamedKey::Space => match self.insert(" ") {
                    Some(edit) => KeyResult::Edit(edit),
                    None => KeyResult::Handled,
                },
                NamedKey::Escape => KeyResult::Blur,
                // Enter and Tab: insert() handles multiline rejection
                NamedKey::Enter => match self.insert("\n") {
                    Some(edit) => KeyResult::Edit(edit),
                    None => KeyResult::Ignored,
                },
                NamedKey::Tab => match self.insert("    ") {
                    Some(edit) => KeyResult::Edit(edit),
                    None => KeyResult::Ignored,
                },
                _ => KeyResult::Ignored,
            },
            _ => KeyResult::Ignored,
        }
    }
}
