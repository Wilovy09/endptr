use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Margin, Rect},
    style::{Color, Style},
    symbols::border::ROUNDED,
    text::Span,
    widgets::{Block, Clear, Paragraph, Widget, Wrap},
};

use crate::{components::centered_rect, widgets::input::TextInput};

/// All modal dialogs the app can show.
#[derive(Debug)]
pub enum Modal {
    SaveRequest {
        name_input: TextInput,
        desc_input: TextInput,
        /// 0 = name, 1 = description
        field: u8,
    },
    AddSecret {
        key_input: TextInput,
        val_input: TextInput,
        /// 0 = editing key, 1 = editing value
        field: u8,
    },
    EditSecret {
        key: String,
        val_input: TextInput,
    },
    ConfirmDelete {
        message: String,
    },
    RequestInfo {
        description: String,
        /// (key, value) pairs of secrets referenced in this request
        secrets_used: Vec<(String, String)>,
        show_secrets: bool,
        scroll: u16,
    },
}

impl Widget for &Modal {
    fn render(self, area: Rect, buf: &mut Buffer) {
        match self {
            Modal::SaveRequest {
                name_input,
                desc_input,
                field,
            } => {
                render_two_inputs(
                    area,
                    buf,
                    " Save Request ",
                    "Name:",
                    name_input,
                    "Description: (optional)",
                    desc_input,
                    *field,
                );
            }
            Modal::AddSecret {
                key_input,
                val_input,
                field,
            } => {
                render_two_inputs(
                    area,
                    buf,
                    " Add Secret ",
                    "Key:",
                    key_input,
                    "Value:",
                    val_input,
                    *field,
                );
            }
            Modal::EditSecret { key, val_input } => {
                let label = format!(" Edit: {} ", key);
                render_single_input(area, buf, &label, "Value:", val_input, true);
            }
            Modal::ConfirmDelete { message } => {
                render_confirm(area, buf, message);
            }
            Modal::RequestInfo {
                description,
                secrets_used,
                show_secrets,
                scroll,
            } => {
                render_request_info(area, buf, description, secrets_used, *show_secrets, *scroll);
            }
        }
    }
}

fn render_single_input(
    area: Rect,
    buf: &mut Buffer,
    title: &str,
    label: &str,
    input: &TextInput,
    focused: bool,
) {
    let modal = centered_rect(50, 20, area);
    Clear.render(modal, buf);

    Block::bordered()
        .border_set(ROUNDED)
        .border_style(Style::new().fg(Color::Yellow))
        .title(title)
        .render(modal, buf);

    let inner = modal.inner(Margin::new(2, 2));
    let rows = Layout::vertical([Constraint::Length(1), Constraint::Length(1)]).split(inner);

    Paragraph::new(Span::styled(label, Style::new().fg(Color::Gray))).render(rows[0], buf);
    Paragraph::new(input.as_line(focused)).render(rows[1], buf);

    let hint = Rect {
        x: modal.x + 1,
        y: modal.y + modal.height - 1,
        width: modal.width - 2,
        height: 1,
    };
    Paragraph::new(Span::styled(
        " Enter: confirm  Esc: cancel ",
        Style::new().fg(Color::DarkGray),
    ))
    .render(hint, buf);
}

fn render_two_inputs(
    area: Rect,
    buf: &mut Buffer,
    title: &str,
    label1: &str,
    input1: &TextInput,
    label2: &str,
    input2: &TextInput,
    active_field: u8,
) {
    let modal = centered_rect(50, 30, area);
    Clear.render(modal, buf);

    Block::bordered()
        .border_set(ROUNDED)
        .border_style(Style::new().fg(Color::Yellow))
        .title(title)
        .render(modal, buf);

    let inner = modal.inner(Margin::new(2, 2));
    let rows = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .split(inner);

    Paragraph::new(Span::styled(label1, Style::new().fg(Color::Gray))).render(rows[0], buf);
    Paragraph::new(input1.as_line(active_field == 0)).render(rows[1], buf);
    Paragraph::new(Span::styled(label2, Style::new().fg(Color::Gray))).render(rows[2], buf);
    Paragraph::new(input2.as_line(active_field == 1)).render(rows[3], buf);

    let hint = Rect {
        x: modal.x + 1,
        y: modal.y + modal.height - 1,
        width: modal.width - 2,
        height: 1,
    };
    Paragraph::new(Span::styled(
        " Tab: next field  Enter: confirm  Esc: cancel ",
        Style::new().fg(Color::DarkGray),
    ))
    .render(hint, buf);
}

fn render_confirm(area: Rect, buf: &mut Buffer, message: &str) {
    let modal = centered_rect(50, 20, area);
    Clear.render(modal, buf);

    Block::bordered()
        .border_set(ROUNDED)
        .border_style(Style::new().fg(Color::Red))
        .title(" Confirm Delete ")
        .render(modal, buf);

    let inner = modal.inner(Margin::new(2, 2));
    Paragraph::new(message)
        .style(Style::new().fg(Color::White))
        .wrap(Wrap { trim: true })
        .render(inner, buf);

    let hint = Rect {
        x: modal.x + 1,
        y: modal.y + modal.height - 1,
        width: modal.width - 2,
        height: 1,
    };
    Paragraph::new(Span::styled(
        " y: confirm  Esc/n: cancel ",
        Style::new().fg(Color::DarkGray),
    ))
    .render(hint, buf);
}

fn render_request_info(
    area: Rect,
    buf: &mut Buffer,
    description: &str,
    secrets_used: &[(String, String)],
    show_secrets: bool,
    scroll: u16,
) {
    let modal = centered_rect(60, 70, area);
    Clear.render(modal, buf);

    Block::bordered()
        .border_set(ROUNDED)
        .border_style(Style::new().fg(Color::Cyan))
        .title(" Request Info ")
        .render(modal, buf);

    let inner = modal.inner(Margin::new(2, 1));

    // Reserve last row for hint bar
    let content_area = Rect {
        x: inner.x,
        y: inner.y,
        width: inner.width,
        height: inner.height.saturating_sub(1),
    };

    // Build lines
    let mut lines: Vec<ratatui::text::Line> = Vec::new();

    // Description section
    lines.push(ratatui::text::Line::from(Span::styled(
        "Description",
        Style::new().fg(Color::Yellow).bold(),
    )));
    lines.push(ratatui::text::Line::from(""));
    if description.is_empty() {
        lines.push(ratatui::text::Line::from(Span::styled(
            "(none)",
            Style::new().fg(Color::DarkGray),
        )));
    } else {
        // Wrap description manually at content width
        let max_w = content_area.width as usize;
        for word_line in description.lines() {
            if word_line.is_empty() {
                lines.push(ratatui::text::Line::from(""));
            } else {
                let mut current = String::new();
                for word in word_line.split_whitespace() {
                    if current.is_empty() {
                        current.push_str(word);
                    } else if current.len() + 1 + word.len() <= max_w {
                        current.push(' ');
                        current.push_str(word);
                    } else {
                        lines.push(ratatui::text::Line::from(Span::styled(
                            current.clone(),
                            Style::new().fg(Color::White),
                        )));
                        current = word.to_string();
                    }
                }
                if !current.is_empty() {
                    lines.push(ratatui::text::Line::from(Span::styled(
                        current,
                        Style::new().fg(Color::White),
                    )));
                }
            }
        }
    }

    lines.push(ratatui::text::Line::from(""));
    lines.push(ratatui::text::Line::from(""));

    // Secrets section
    let toggle_label = if show_secrets { "hide" } else { "show" };
    lines.push(ratatui::text::Line::from(vec![
        Span::styled("Secrets used", Style::new().fg(Color::Yellow).bold()),
        Span::styled(
            format!("  [t: {}]", toggle_label),
            Style::new().fg(Color::DarkGray),
        ),
    ]));
    lines.push(ratatui::text::Line::from(""));

    if secrets_used.is_empty() {
        lines.push(ratatui::text::Line::from(Span::styled(
            "(none detected)",
            Style::new().fg(Color::DarkGray),
        )));
    } else {
        for (key, value) in secrets_used {
            let val_display = if show_secrets {
                value.clone()
            } else {
                "•".repeat(value.len().min(20))
            };
            lines.push(ratatui::text::Line::from(vec![
                Span::styled(format!("{{{{{}}}}} ", key), Style::new().fg(Color::Cyan)),
                Span::styled("= ", Style::new().fg(Color::DarkGray)),
                Span::styled(val_display, Style::new().fg(Color::White)),
            ]));
        }
    }

    Paragraph::new(lines)
        .scroll((scroll, 0))
        .render(content_area, buf);

    // Hint bar
    let hint = Rect {
        x: modal.x + 1,
        y: modal.y + modal.height - 1,
        width: modal.width - 2,
        height: 1,
    };
    Paragraph::new(Span::styled(
        " t: toggle secrets  j/k: scroll  Esc: close ",
        Style::new().fg(Color::DarkGray),
    ))
    .render(hint, buf);
}
