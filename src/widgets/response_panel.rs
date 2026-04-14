use ratatui::{
    buffer::Buffer,
    layout::{Margin, Rect},
    style::{Color, Style, Stylize},
    symbols::border::ROUNDED,
    text::{Line, Span},
    widgets::{Block, Paragraph, ScrollbarState, Widget, Wrap},
};

use crate::models::HttpResponse;

pub struct ResponsePanel<'a> {
    pub response: Option<&'a Result<HttpResponse, String>>,
    pub is_loading: bool,
    pub scroll: u16,
    pub focused: bool,
}

impl<'a> ResponsePanel<'a> {
    pub fn new(
        response: Option<&'a Result<HttpResponse, String>>,
        is_loading: bool,
        scroll: u16,
        focused: bool,
    ) -> Self {
        Self {
            response,
            is_loading,
            scroll,
            focused,
        }
    }
}

impl<'a> Widget for ResponsePanel<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let focus_color = if self.focused { Color::Yellow } else { Color::White };

        // ── Status line ───────────────────────────────────────────────────────
        let title: Line = match &self.response {
            _ if self.is_loading => Line::from(vec![Span::styled(
                " ⏳ Sending... ",
                Style::new().fg(Color::Yellow),
            )]),
            Some(Ok(r)) => Line::from(vec![
                Span::raw(" "),
                Span::styled(
                    format!("{} {}", r.status, r.status_text),
                    Style::new().fg(r.status_color()).bold(),
                ),
                Span::styled(
                    format!("  {}ms ", r.time_ms),
                    Style::new().fg(Color::DarkGray),
                ),
            ]),
            Some(Err(e)) => Line::from(vec![Span::styled(
                format!(" ✗ {} ", e),
                Style::new().fg(Color::Red),
            )]),
            None => Line::from(Span::styled(
                " Response ",
                Style::new().fg(Color::DarkGray),
            )),
        };

        let block = Block::bordered()
            .border_set(ROUNDED)
            .border_style(Style::new().fg(focus_color))
            .title(title);

        let inner = area.inner(Margin::new(1, 1));
        block.render(area, buf);

        // ── Body ──────────────────────────────────────────────────────────────
        match &self.response {
            Some(Ok(r)) => {
                // Try to pretty-print JSON
                let display_body = if let Ok(v) = serde_json::from_str::<serde_json::Value>(&r.body)
                {
                    serde_json::to_string_pretty(&v).unwrap_or(r.body.clone())
                } else {
                    r.body.clone()
                };

                Paragraph::new(display_body)
                    .scroll((self.scroll, 0))
                    .wrap(Wrap { trim: false })
                    .render(inner, buf);
            }
            Some(Err(e)) => {
                Paragraph::new(e.as_str())
                    .style(Style::new().fg(Color::Red))
                    .wrap(Wrap { trim: false })
                    .render(inner, buf);
            }
            None if !self.is_loading => {
                Paragraph::new(Span::styled(
                    "Press ▶  or <Enter> in URL bar to send request",
                    Style::new().fg(Color::DarkGray).italic(),
                ))
                .render(inner, buf);
            }
            _ => {}
        }

        // Scroll hint
        if self.focused {
            if let Some(Ok(_)) = &self.response {
                if inner.height > 1 {
                    let hint_area = Rect {
                        x: inner.x,
                        y: inner.y + inner.height - 1,
                        width: inner.width,
                        height: 1,
                    };
                    Paragraph::new(Span::styled(
                        " j/k: scroll  h: toggle headers ",
                        Style::new().fg(Color::DarkGray),
                    ))
                    .render(hint_area, buf);
                }
            }
        }
    }
}
