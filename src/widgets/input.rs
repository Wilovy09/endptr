use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
};

/// Multi-line text area with cursor support.
#[derive(Debug, Default, Clone)]
pub struct TextArea {
    pub value: String,
    pub cursor: usize, // char index into flat string
}

impl TextArea {
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

    pub fn insert_newline(&mut self) {
        self.insert('\n');
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

    pub fn move_up(&mut self) {
        let (line, col) = self.cursor_pos();
        if line == 0 {
            self.cursor = 0;
            return;
        }
        let lines: Vec<&str> = self.value.split('\n').collect();
        let target_col = col.min(lines[line - 1].chars().count());
        self.cursor = self.line_col_to_cursor(line - 1, target_col);
    }

    pub fn move_down(&mut self) {
        let (line, col) = self.cursor_pos();
        let lines: Vec<&str> = self.value.split('\n').collect();
        if line + 1 >= lines.len() {
            self.cursor = self.char_len();
            return;
        }
        let target_col = col.min(lines[line + 1].chars().count());
        self.cursor = self.line_col_to_cursor(line + 1, target_col);
    }

    pub fn move_to_line_start(&mut self) {
        let (line, _) = self.cursor_pos();
        self.cursor = self.line_col_to_cursor(line, 0);
    }

    pub fn move_to_line_end(&mut self) {
        let (line, _) = self.cursor_pos();
        let lines: Vec<&str> = self.value.split('\n').collect();
        let line_len = lines[line].chars().count();
        self.cursor = self.line_col_to_cursor(line, line_len);
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

    fn cursor_pos(&self) -> (usize, usize) {
        let mut line = 0usize;
        let mut col = 0usize;
        for (i, c) in self.value.chars().enumerate() {
            if i == self.cursor {
                break;
            }
            if c == '\n' {
                line += 1;
                col = 0;
            } else {
                col += 1;
            }
        }
        (line, col)
    }

    fn line_col_to_cursor(&self, target_line: usize, target_col: usize) -> usize {
        let mut line = 0usize;
        let mut col = 0usize;
        for (i, c) in self.value.chars().enumerate() {
            if line == target_line && col == target_col {
                return i;
            }
            if c == '\n' {
                if line == target_line {
                    // target_col was past end of line — clamp to newline position
                    return i;
                }
                line += 1;
                col = 0;
            } else {
                col += 1;
            }
        }
        self.value.chars().count()
    }

    /// Render as `Text` (multiple lines) with cursor shown when `focused`.
    pub fn as_text(&self, focused: bool) -> ratatui::text::Text<'_> {
        use ratatui::text::{Line, Span, Text};

        let lines_str: Vec<&str> = self.value.split('\n').collect();
        let (cursor_line, cursor_col) = if focused {
            self.cursor_pos()
        } else {
            (usize::MAX, usize::MAX)
        };

        let rendered: Vec<Line> = lines_str
            .iter()
            .enumerate()
            .map(|(li, &line_text)| {
                if li != cursor_line {
                    return Line::from(line_text.to_string());
                }
                let chars: Vec<char> = line_text.chars().collect();
                let mut spans: Vec<Span> = Vec::with_capacity(chars.len() + 1);
                for (ci, c) in chars.iter().enumerate() {
                    if ci == cursor_col {
                        spans.push(Span::styled(
                            c.to_string(),
                            Style::default().bg(Color::White).fg(Color::Black),
                        ));
                    } else {
                        spans.push(Span::raw(c.to_string()));
                    }
                }
                if cursor_col >= chars.len() {
                    spans.push(Span::styled(
                        " ",
                        Style::default().bg(Color::White).fg(Color::Black),
                    ));
                }
                Line::from(spans)
            })
            .collect();

        Text::from(rendered)
    }
}

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
