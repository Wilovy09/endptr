use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Margin, Rect},
    style::{Color, Style},
    symbols::border::ROUNDED,
    text::{Line, Span, Text},
    widgets::{Block, List, ListItem, ListState, Paragraph, StatefulWidget, Widget},
};

use crate::{
    highlight,
    models::{AuthType, QueryParam},
    widgets::input::{TextArea, TextInput},
};

// ── Tabs ──────────────────────────────────────────────────────────────────────

#[derive(Debug, Default, PartialEq, Clone)]
pub enum RequestTab {
    #[default]
    Body,
    Headers,
    Auth,
    Params,
}

impl RequestTab {
    pub fn index(&self) -> usize {
        match self {
            RequestTab::Body => 0,
            RequestTab::Headers => 1,
            RequestTab::Auth => 2,
            RequestTab::Params => 3,
        }
    }

    pub fn next(&self) -> RequestTab {
        match self {
            RequestTab::Body => RequestTab::Headers,
            RequestTab::Headers => RequestTab::Auth,
            RequestTab::Auth => RequestTab::Params,
            RequestTab::Params => RequestTab::Body,
        }
    }

    pub fn prev(&self) -> RequestTab {
        match self {
            RequestTab::Body => RequestTab::Params,
            RequestTab::Headers => RequestTab::Body,
            RequestTab::Auth => RequestTab::Headers,
            RequestTab::Params => RequestTab::Auth,
        }
    }
}

// ── State ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Default)]
pub struct RequestPanelState {
    pub tab: RequestTab,

    // Headers
    pub header_list: ListState,

    // Auth — inline text inputs (owned here, synced to App.current.auth on key events)
    pub auth_type: AuthType,
    pub auth_username: TextInput,
    pub auth_password: TextInput,
    pub auth_token: TextInput,
    /// 0 = type selector, 1 = first field, 2 = second field (password for basic)
    pub auth_field: u8,

    // Params
    pub params_list: ListState,
}

impl RequestPanelState {
    pub fn auth_field_count(&self) -> u8 {
        match self.auth_type {
            AuthType::None => 0,
            AuthType::Basic => 2,
            _ => 1,
        }
    }

    pub fn auth_next_field(&mut self) {
        let max = self.auth_field_count();
        if max == 0 {
            self.auth_field = 0;
        } else {
            self.auth_field = (self.auth_field + 1) % (max + 1);
        }
    }

    pub fn auth_prev_field(&mut self) {
        let max = self.auth_field_count();
        if max == 0 {
            self.auth_field = 0;
        } else {
            self.auth_field = if self.auth_field == 0 {
                max
            } else {
                self.auth_field - 1
            };
        }
    }

    /// Active TextInput for the current auth field (1 or 2).
    pub fn active_auth_input(&mut self) -> Option<&mut TextInput> {
        match (&self.auth_type, self.auth_field) {
            (AuthType::Basic, 1) => Some(&mut self.auth_username),
            (AuthType::Basic, 2) => Some(&mut self.auth_password),
            (AuthType::Bearer | AuthType::OAuth2 | AuthType::JWT, 1) => Some(&mut self.auth_token),
            _ => None,
        }
    }
}

// ── Widget ────────────────────────────────────────────────────────────────────

pub struct RequestPanel<'a> {
    pub body_input: &'a TextArea,
    pub headers: &'a [(String, String)],
    pub params: &'a [QueryParam],
    pub focused: bool,
}

impl<'a> RequestPanel<'a> {
    pub fn new(
        body_input: &'a TextArea,
        headers: &'a [(String, String)],
        params: &'a [QueryParam],
        focused: bool,
    ) -> Self {
        Self {
            body_input,
            headers,
            params,
            focused,
        }
    }
}

impl<'a> StatefulWidget for RequestPanel<'a> {
    type State = RequestPanelState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let focus_color = if self.focused {
            Color::Yellow
        } else {
            Color::White
        };

        // ── Tab title integrated into border ─────────────────────────────────
        let tab_names = ["Body", "Headers", "Auth", "Params"];
        let mut title_spans: Vec<Span> = Vec::new();
        for (i, name) in tab_names.iter().enumerate() {
            let style = if i == state.tab.index() {
                Style::new().fg(Color::Yellow).bold()
            } else {
                Style::new().fg(Color::DarkGray)
            };
            title_spans.push(Span::styled(*name, style));
            if i < tab_names.len() - 1 {
                title_spans.push(Span::raw(" ─ "));
            }
        }

        // ── Content block ─────────────────────────────────────────────────────
        let block = Block::bordered()
            .border_set(ROUNDED)
            .border_style(Style::new().fg(focus_color))
            .title(Line::from(title_spans));
        let inner = area.inner(Margin::new(1, 1));
        block.render(area, buf);

        match state.tab {
            RequestTab::Body => render_body(inner, buf, self.body_input, self.focused),
            RequestTab::Headers => render_headers(inner, buf, self.headers, self.focused, state),
            RequestTab::Auth => render_auth(inner, buf, self.focused, state),
            RequestTab::Params => render_params(inner, buf, self.params, self.focused, state),
        }
    }
}

// ── Body ──────────────────────────────────────────────────────────────────────

fn render_body(area: Rect, buf: &mut Buffer, input: &TextArea, focused: bool) {
    if focused {
        Paragraph::new(input.as_text(true)).render(area, buf);
    } else if input.value.trim().is_empty() {
        Paragraph::new(Span::styled(
            "Empty body  (focus + type to edit,  y to copy)",
            Style::new().fg(Color::DarkGray).italic(),
        ))
        .render(area, buf);
    } else {
        let lines = highlight::highlight_json_str(&input.value);
        Paragraph::new(Text::from(lines))
            .wrap(ratatui::widgets::Wrap { trim: false })
            .render(area, buf);
    }
}

// ── Headers ───────────────────────────────────────────────────────────────────

fn render_headers(
    area: Rect,
    buf: &mut Buffer,
    headers: &[(String, String)],
    focused: bool,
    state: &mut RequestPanelState,
) {
    let items: Vec<ListItem> = headers
        .iter()
        .map(|(k, v)| {
            ListItem::new(Line::from(vec![
                Span::styled(k.clone(), Style::new().fg(Color::Cyan)),
                Span::raw(": "),
                Span::raw(v.clone()),
            ]))
        })
        .collect();

    let list = List::new(items).highlight_style(Style::new().bg(Color::DarkGray));
    StatefulWidget::render(list, area, buf, &mut state.header_list);

    if focused && area.height > 1 {
        Paragraph::new(Span::styled(
            " n: add  d: delete ",
            Style::new().fg(Color::DarkGray),
        ))
        .render(
            Rect {
                x: area.x,
                y: area.y + area.height - 1,
                width: area.width,
                height: 1,
            },
            buf,
        );
    }
}

// ── Auth ──────────────────────────────────────────────────────────────────────

fn render_auth(area: Rect, buf: &mut Buffer, focused: bool, state: &mut RequestPanelState) {
    if area.height < 3 {
        return;
    }

    let rows = Layout::vertical([
        Constraint::Length(1), // type row
        Constraint::Length(1), // spacer
        Constraint::Length(1), // field 1 label
        Constraint::Length(1), // field 1 input
        Constraint::Length(1), // field 2 label (basic only)
        Constraint::Length(1), // field 2 input (basic only)
    ])
    .split(area);

    // ── Auth type selector ────────────────────────────────────────────────────
    let type_selected = focused && state.auth_field == 0;
    let type_style = if type_selected {
        Style::new().fg(Color::Black).bg(Color::Yellow)
    } else {
        Style::new().fg(Color::Yellow)
    };

    let all_types = AuthType::all();
    let type_line: Line = Line::from(
        all_types
            .iter()
            .flat_map(|t| {
                let active = *t == state.auth_type;
                let style = if active && type_selected {
                    Style::new().fg(Color::Black).bg(Color::Yellow).bold()
                } else if active {
                    Style::new().fg(Color::Yellow).bold()
                } else {
                    Style::new().fg(Color::DarkGray)
                };
                vec![
                    Span::styled(format!(" {} ", t.label()), style),
                    Span::raw(" "),
                ]
            })
            .collect::<Vec<_>>(),
    );
    Paragraph::new(type_line).render(rows[0], buf);

    // ── Fields ────────────────────────────────────────────────────────────────
    match &state.auth_type {
        AuthType::None => {
            Paragraph::new(Span::styled(
                "No authentication",
                Style::new().fg(Color::DarkGray).italic(),
            ))
            .render(rows[2], buf);
        }

        AuthType::Basic => {
            // Username
            let user_focused = focused && state.auth_field == 1;
            Paragraph::new(Span::styled("Username:", Style::new().fg(Color::Gray)))
                .render(rows[2], buf);
            Paragraph::new(state.auth_username.as_line(user_focused)).render(rows[3], buf);

            // Password
            let pass_focused = focused && state.auth_field == 2;
            Paragraph::new(Span::styled("Password:", Style::new().fg(Color::Gray)))
                .render(rows[4], buf);
            // Mask password display
            let pass_display = if pass_focused {
                state.auth_password.as_line(true)
            } else {
                Line::from("*".repeat(state.auth_password.char_len()))
            };
            Paragraph::new(pass_display).render(rows[5], buf);
        }

        auth_type => {
            let label = match auth_type {
                AuthType::OAuth2 => "Access Token:",
                AuthType::JWT => "JWT Token:",
                _ => "Token:",
            };
            let tok_focused = focused && state.auth_field == 1;
            Paragraph::new(Span::styled(label, Style::new().fg(Color::Gray))).render(rows[2], buf);
            Paragraph::new(state.auth_token.as_line(tok_focused)).render(rows[3], buf);
        }
    }

    // Hint
    if focused && area.height > 1 {
        Paragraph::new(Span::styled(
            " ←/→: change type  Tab: next field ",
            Style::new().fg(Color::DarkGray),
        ))
        .render(
            Rect {
                x: area.x,
                y: area.y + area.height - 1,
                width: area.width,
                height: 1,
            },
            buf,
        );
    }
}

// ── Params ────────────────────────────────────────────────────────────────────

fn render_params(
    area: Rect,
    buf: &mut Buffer,
    params: &[QueryParam],
    focused: bool,
    state: &mut RequestPanelState,
) {
    if area.height < 2 {
        return;
    }

    // Column layout: enabled(3) | key(40%) | value(rest)
    let col_w = area.width.saturating_sub(3);
    let key_w = col_w * 40 / 100;
    let val_w = col_w.saturating_sub(key_w);

    // Header row
    if area.height >= 2 {
        let header_area = Rect {
            x: area.x,
            y: area.y,
            width: area.width,
            height: 1,
        };
        Paragraph::new(Line::from(vec![
            Span::styled("On ", Style::new().fg(Color::DarkGray).bold()),
            Span::styled(
                format!("{:<width$}", "Key", width = key_w as usize),
                Style::new().fg(Color::DarkGray).bold(),
            ),
            Span::styled("Value", Style::new().fg(Color::DarkGray).bold()),
        ]))
        .render(header_area, buf);
    }

    let list_area = Rect {
        x: area.x,
        y: area.y + 1,
        width: area.width,
        height: area.height.saturating_sub(2),
    };

    let items: Vec<ListItem> = params
        .iter()
        .map(|p| {
            let check = if p.enabled {
                Span::styled("✓  ", Style::new().fg(Color::Green))
            } else {
                Span::styled("·  ", Style::new().fg(Color::DarkGray))
            };
            let key = Span::styled(
                format!("{:<width$}", p.key, width = key_w as usize),
                Style::new().fg(Color::Cyan),
            );
            let val = Span::raw(p.value.clone());
            ListItem::new(Line::from(vec![check, key, val]))
        })
        .collect();

    let list = List::new(items).highlight_style(Style::new().bg(Color::DarkGray));
    StatefulWidget::render(list, list_area, buf, &mut state.params_list);

    if focused && area.height > 1 {
        Paragraph::new(Span::styled(
            " n: add  d: delete  Space: toggle ",
            Style::new().fg(Color::DarkGray),
        ))
        .render(
            Rect {
                x: area.x,
                y: area.y + area.height - 1,
                width: area.width,
                height: 1,
            },
            buf,
        );
    }
}
