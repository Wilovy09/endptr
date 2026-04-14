use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Margin, Rect},
    style::{Color, Style, Stylize},
    symbols::border::ROUNDED,
    text::{Line, Span},
    widgets::{Block, Paragraph, Widget},
};

use crate::{models::HttpMethod, widgets::input::TextInput};

pub struct Toolbar<'a> {
    pub method: &'a HttpMethod,
    pub url_input: &'a TextInput,
    pub is_loading: bool,
    pub focus: ToolbarFocus,
}

#[derive(Debug, PartialEq, Clone, Default)]
pub enum ToolbarFocus {
    #[default]
    None,
    Method,
    Url,
}

impl<'a> Toolbar<'a> {
    pub fn new(
        method: &'a HttpMethod,
        url_input: &'a TextInput,
        is_loading: bool,
        focus: ToolbarFocus,
    ) -> Self {
        Self {
            method,
            url_input,
            is_loading,
            focus,
        }
    }
}

impl<'a> Widget for Toolbar<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // [Method 10%] [URL 90%]
        let cols = Layout::horizontal([
            Constraint::Percentage(10),
            Constraint::Percentage(90),
        ])
        .split(area);

        // ── Method selector ───────────────────────────────────────────────────
        {
            let focused = self.focus == ToolbarFocus::Method;
            let border_color = if focused { Color::Yellow } else { Color::White };
            let block = Block::bordered()
                .border_set(ROUNDED)
                .border_style(Style::new().fg(border_color));

            let inner = cols[0].inner(Margin::new(1, 1));
            block.render(cols[0], buf);

            Paragraph::new(Line::from(vec![Span::styled(
                self.method.as_str(),
                Style::new().fg(self.method.color()).bold(),
            )]))
            .centered()
            .render(inner, buf);
        }

        // ── URL input ─────────────────────────────────────────────────────────
        {
            let focused = self.focus == ToolbarFocus::Url;
            let border_color = if focused { Color::Yellow } else { Color::White };
            let block = Block::bordered()
                .border_set(ROUNDED)
                .border_style(Style::new().fg(border_color))
                .title(" URL ");

            let inner = cols[1].inner(Margin::new(1, 1));
            block.render(cols[1], buf);

            // Scroll view: show tail of URL when it's long
            let line = self.url_input.as_line(focused);
            Paragraph::new(line).render(inner, buf);
        }

    }
}
