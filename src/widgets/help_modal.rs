use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Margin, Rect},
    style::{Color, Style},
    symbols::border::ROUNDED,
    text::Text,
    widgets::{Block, Clear, Paragraph, StatefulWidget, Widget},
};

use crate::{components::centered_rect, keybinds::KeyMap};

#[derive(Debug, Default)]
pub struct HelpModalState {
    pub is_open: bool,
}

impl HelpModalState {
    pub fn toggle(&mut self) {
        self.is_open = !self.is_open;
    }

    #[allow(dead_code)]
    pub fn open(&mut self) {
        self.is_open = true;
    }

    #[allow(dead_code)]
    pub fn close(&mut self) {
        self.is_open = false;
    }
}

pub struct HelpModal<'a> {
    pub keymap: &'a KeyMap,
}

impl<'a> HelpModal<'a> {
    pub fn new(keymap: &'a KeyMap) -> Self {
        Self { keymap }
    }
}

impl<'a> StatefulWidget for HelpModal<'a> {
    type State = HelpModalState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        if !state.is_open {
            return;
        }

        Clear.render(area, buf);

        let modal_area = centered_rect(60, 60, area);

        Block::bordered()
            .border_set(ROUNDED)
            .title(" Key Bindings ")
            .render(modal_area, buf);

        let inner_area = modal_area.inner(Margin::new(3, 3));

        let help_content = self.create_help_content();

        Paragraph::new(help_content)
            .style(Style::default().fg(Color::White))
            .alignment(Alignment::Left)
            .render(inner_area, buf);
    }
}

impl<'a> HelpModal<'a> {
    fn create_help_content(&self) -> Text<'_> {
        let mut formatted_lines = Vec::new();

        for (action, key) in self.keymap.get_bindings() {
            let mut key_strings = Vec::new();

            for (code, mods) in key.iter() {
                let key_str = match code {
                    KeyCode::Char(c) => c.to_string(),
                    other => format!("{:?}", other),
                };

                if *mods == KeyModifiers::NONE {
                    key_strings.push(key_str);
                } else {
                    let mods_str = format!("{:?}", mods)
                        .replace("KeyModifiers(", "")
                        .replace(')', "");
                    key_strings.push(format!("<{mods_str} + {key_str}>"));
                }
            }

            let combined_keys = key_strings.join(" or ");
            formatted_lines.push((combined_keys, action));
        }

        let max_key_width = formatted_lines
            .iter()
            .map(|(keys, _)| keys.len())
            .max()
            .unwrap_or(0);

        let mut lines = Vec::new();
        for (keys, action) in formatted_lines {
            let formatted_line = format!("{:<width$} - {}\n", keys, action, width = max_key_width);
            lines.push(formatted_line);
        }

        Text::from(lines.join(""))
    }
}
