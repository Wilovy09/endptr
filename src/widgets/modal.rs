use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Margin, Rect},
    style::{Color, Style, Stylize},
    symbols::border::ROUNDED,
    text::{Line, Span},
    widgets::{Block, Clear, Paragraph, Widget},
};

use crate::{components::centered_rect, widgets::input::TextInput};

/// All modal dialogs the app can show.
#[derive(Debug)]
pub enum Modal {
    SaveRequest {
        name_input: TextInput,
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
}

impl Widget for &Modal {
    fn render(self, area: Rect, buf: &mut Buffer) {
        match self {
            Modal::SaveRequest { name_input } => {
                render_single_input(area, buf, " Save Request ", "Name:", name_input, true);
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
        .wrap(ratatui::widgets::Wrap { trim: true })
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
