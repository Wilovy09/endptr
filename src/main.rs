use crossterm::event::{self, Event};
use ratatui::DefaultTerminal;

use app::App;

use crate::app::AppUi;

mod app;
mod components;
mod keybinds;
mod widgets;

fn main() -> std::io::Result<()> {
    let terminal = ratatui::init();
    let result = run(terminal);
    ratatui::restore();
    result
}

fn run(mut terminal: DefaultTerminal) -> std::io::Result<()> {
    let mut state = App {
        ..Default::default()
    };

    let keymap = keybinds::KeyMap::new();

    loop {
        terminal.draw(|frame| {
            frame.render_stateful_widget(AppUi::new(&keymap), frame.area(), &mut state);
        })?;

        if let Event::Key(key_event) = event::read()? {
            if keymap.quit.matches(&key_event) {
                break Ok(());
            } else if keymap.help.matches(&key_event) {
                state.help_modal_state.toggle();
            }
        }
    }
}
