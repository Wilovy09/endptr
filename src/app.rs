use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    symbols,
    widgets::{Block, Clear, StatefulWidget, Widget},
};

use crate::widgets::{HelpModal, HelpModalState, Sidebar, SidebarState};

#[derive(Debug, Default)]
pub struct App {
    pub sidebar_state: SidebarState,
    pub help_modal_state: HelpModalState,
}

pub struct AppUi<'a> {
    pub keymap: &'a crate::keybinds::KeyMap,
}

impl<'a> AppUi<'a> {
    pub fn new(keymap: &'a crate::keybinds::KeyMap) -> Self {
        Self { keymap }
    }
}

impl<'a> StatefulWidget for AppUi<'a> {
    type State = App;

    /// `area` es la region donde debes renderizar
    /// `buf` es toda la terminal
    /// `state` pues state
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        Clear.render(area, buf);

        let full_area = Rect {
            x: 0,
            y: 0,
            width: buf.area.width,
            height: buf.area.height,
        };

        let area = Layout::horizontal(vec![Constraint::Percentage(20), Constraint::Percentage(80)])
            .split(area);

        // Sidebar
        Sidebar.render(area[0], buf, &mut state.sidebar_state);

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

        HelpModal::new(self.keymap).render(full_area, buf, &mut state.help_modal_state);
    }
}
