use std::time::Duration;

use crossterm::event::{self, Event};
use ratatui::DefaultTerminal;

use app::{App, AppUi};

mod app;
mod components;
mod highlight;
mod http_client;
mod keybinds;
mod models;
mod storage;
mod widgets;
mod workflow;

fn main() -> std::io::Result<()> {
    let terminal = ratatui::init();
    let result = run(terminal);
    ratatui::restore();
    result
}

fn run(mut terminal: DefaultTerminal) -> std::io::Result<()> {
    let mut state = App::default();
    let keymap = keybinds::KeyMap::new();

    loop {
        // Poll async operations before drawing
        state.poll_http();
        state.poll_workflow();

        terminal.draw(|frame| {
            frame.render_stateful_widget(AppUi::new(&keymap), frame.area(), &mut state);
        })?;

        // 50ms timeout so HTTP responses are picked up promptly
        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key_event) = event::read()? {
                if state.handle_key(key_event, &keymap) {
                    break Ok(());
                }
            }
        }
    }
}
