use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Margin, Rect},
    style::{Style, Stylize},
    symbols,
    text::Text,
    widgets::{Block, Clear, StatefulWidget, Widget},
};

#[derive(Debug, Default)]
pub struct App {
    pub counter: u8,
    pub exit: bool,
}

// Estructura unitaria???????????
// Estructura sin valor/tamaño
pub struct AppUi;

impl StatefulWidget for AppUi {
    type State = App;

    /// `area` es la region donde debes renderizar
    /// `buf` es toda la terminal
    /// `state` pues state
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        Clear.render(area, buf);

        let area = Layout::horizontal(vec![Constraint::Percentage(20), Constraint::Percentage(80)])
            .split(area);

        // Sidebar
        {
            let area = Layout::vertical(vec![Constraint::Length(3), Constraint::Percentage(100)])
                .split(area[0]);

            Block::bordered()
                .title("Requests")
                .style(Style::new().bold())
                .border_set(symbols::border::ROUNDED)
                .render(area[1], buf);

            let title_secrets =
                Layout::horizontal(vec![Constraint::Percentage(100), Constraint::Length(11)])
                    .split(area[0]);

            let text_area = title_secrets[0].inner(Margin::new(1, 1));
            Text::from("End*").centered().bold().render(text_area, buf);
            Block::bordered()
                .border_set(symbols::border::ROUNDED)
                .render(title_secrets[0], buf);

            let text_area = title_secrets[1].inner(Margin::new(1, 1));
            Text::from("Secrets")
                .centered()
                .bold()
                .render(text_area, buf);
            Block::bordered()
                .border_set(symbols::border::ROUNDED)
                .render(title_secrets[1], buf);
        }

        // Main && Toolbar
        {
            let area =
                Layout::vertical(vec![Constraint::Percentage(5), Constraint::Percentage(95)])
                    .split(area[1]);
            Block::bordered()
                .border_set(symbols::border::ROUNDED)
                .render(area[0], buf);
            Block::bordered()
                .border_set(symbols::border::ROUNDED)
                .render(area[1], buf);

            let toolbar = Layout::horizontal(vec![
                Constraint::Percentage(10),
                Constraint::Percentage(85),
                Constraint::Percentage(5),
            ])
            .split(area[0]);
            Block::bordered()
                .border_set(symbols::border::ROUNDED)
                .render(toolbar[0], buf);
            Block::bordered()
                .border_set(symbols::border::ROUNDED)
                .render(toolbar[1], buf);
            Block::bordered()
                .border_set(symbols::border::ROUNDED)
                .render(toolbar[2], buf);
        }
    }
}
