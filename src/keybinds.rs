use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone)]
pub struct KeyBinding {
    pub keys: Vec<(KeyCode, KeyModifiers)>,
    pub description: &'static str,
}

impl KeyBinding {
    pub fn new(description: &'static str, keys: Vec<(KeyCode, KeyModifiers)>) -> Self {
        Self { keys, description }
    }

    pub fn matches(&self, event: &KeyEvent) -> bool {
        self.keys
            .iter()
            .any(|(code, mods)| *code == event.code && *mods == event.modifiers)
    }
}

#[derive(Debug)]
pub struct KeyMap {
    pub quit: KeyBinding,
    pub help: KeyBinding,
}

impl KeyMap {
    pub fn new() -> Self {
        Self {
            quit: KeyBinding::new(
                "Exit program",
                vec![
                    (KeyCode::Char('q'), KeyModifiers::NONE),
                    (KeyCode::Esc, KeyModifiers::NONE),
                ],
            ),
            help: KeyBinding::new("Show help", vec![(KeyCode::Char('?'), KeyModifiers::NONE)]),
        }
    }

    pub fn get_bindings(&self) -> Vec<(&'static str, Vec<(KeyCode, KeyModifiers)>)> {
        vec![
            (self.quit.description, self.quit.keys.clone()),
            (self.help.description, self.help.keys.clone()),
        ]
    }
}
