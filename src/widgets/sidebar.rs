use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Margin, Rect},
    style::{Style, Stylize},
    symbols::border::ROUNDED,
    text::Text,
    widgets::{Block, StatefulWidget, Widget},
};

#[derive(Debug, Default)]
pub struct SidebarState {}

#[allow(dead_code)]
pub struct Sidebar;

impl StatefulWidget for Sidebar {
    type State = SidebarState;

    fn render(self, area: Rect, buf: &mut Buffer, _state: &mut Self::State) {
        let layoout = Layout::vertical(vec![Constraint::Length(3), Constraint::Min(0)]).split(area);

        let text_area = layoout[0].inner(Margin::new(1, 1));
        Text::from("Secrets")
            .centered()
            .bold()
            .render(text_area, buf);

        Block::bordered()
            .border_set(ROUNDED)
            .render(layoout[0], buf);

        Block::bordered()
            .title("Requests")
            .style(Style::new().bold())
            .border_set(ROUNDED)
            .render(layoout[1], buf);
    }
}
