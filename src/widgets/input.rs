use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
};

/// Single-line text input with cursor support.
#[derive(Debug, Default, Clone)]
pub struct TextInput {
    pub value: String,
    pub cursor: usize, // char index
}

impl TextInput {
    pub fn new(value: impl Into<String>) -> Self {
        let value = value.into();
        let cursor = value.chars().count();
        Self { value, cursor }
    }

    pub fn insert(&mut self, c: char) {
        let byte = self.char_to_byte(self.cursor);
        self.value.insert(byte, c);
        self.cursor += 1;
    }

    pub fn delete_before(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
            let byte = self.char_to_byte(self.cursor);
            self.value.remove(byte);
        }
    }

    pub fn delete_after(&mut self) {
        let byte = self.char_to_byte(self.cursor);
        if byte < self.value.len() {
            self.value.remove(byte);
        }
    }

    pub fn move_left(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    pub fn move_right(&mut self) {
        if self.cursor < self.char_len() {
            self.cursor += 1;
        }
    }

    pub fn move_to_start(&mut self) {
        self.cursor = 0;
    }

    pub fn move_to_end(&mut self) {
        self.cursor = self.char_len();
    }

    pub fn clear(&mut self) {
        self.value.clear();
        self.cursor = 0;
    }

    pub fn char_len(&self) -> usize {
        self.value.chars().count()
    }

    fn char_to_byte(&self, char_idx: usize) -> usize {
        self.value
            .char_indices()
            .nth(char_idx)
            .map(|(i, _)| i)
            .unwrap_or(self.value.len())
    }

    /// Render as a `Line` with cursor shown when `focused`.
    pub fn as_line(&self, focused: bool) -> Line<'_> {
        if !focused {
            return Line::from(self.value.clone());
        }

        let chars: Vec<char> = self.value.chars().collect();
        let mut spans: Vec<Span> = Vec::with_capacity(chars.len() + 1);

        for (i, c) in chars.iter().enumerate() {
            if i == self.cursor {
                spans.push(Span::styled(
                    c.to_string(),
                    Style::default().bg(Color::White).fg(Color::Black),
                ));
            } else {
                spans.push(Span::raw(c.to_string()));
            }
        }

        // Cursor past end
        if self.cursor >= chars.len() {
            spans.push(Span::styled(
                " ",
                Style::default().bg(Color::White).fg(Color::Black),
            ));
        }

        Line::from(spans)
    }
}
