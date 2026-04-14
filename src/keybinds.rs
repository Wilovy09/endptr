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
    pub send: KeyBinding,
    pub save: KeyBinding,
    pub tab_next: KeyBinding,
    pub tab_prev: KeyBinding,
    pub up: KeyBinding,
    pub down: KeyBinding,
    pub new_item: KeyBinding,
    pub delete_item: KeyBinding,
    pub toggle_section: KeyBinding,
    pub confirm: KeyBinding,
    pub cancel: KeyBinding,
    pub method_next: KeyBinding,
    pub method_prev: KeyBinding,
    pub new_folder: KeyBinding,
    pub copy: KeyBinding,
}

impl KeyMap {
    pub fn new() -> Self {
        Self {
            quit: KeyBinding::new("Quit", vec![(KeyCode::Char('q'), KeyModifiers::CONTROL)]),
            help: KeyBinding::new(
                "Toggle help",
                vec![(KeyCode::Char('?'), KeyModifiers::NONE)],
            ),
            send: KeyBinding::new(
                "Send request",
                vec![
                    (KeyCode::F(5), KeyModifiers::NONE),
                    (KeyCode::Enter, KeyModifiers::CONTROL),
                ],
            ),
            save: KeyBinding::new(
                "Save request",
                vec![(KeyCode::Char('s'), KeyModifiers::CONTROL)],
            ),
            tab_next: KeyBinding::new("Focus next panel", vec![(KeyCode::Tab, KeyModifiers::NONE)]),
            tab_prev: KeyBinding::new(
                "Focus previous panel",
                vec![(KeyCode::BackTab, KeyModifiers::SHIFT)],
            ),
            up: KeyBinding::new(
                "Move up",
                vec![
                    (KeyCode::Char('k'), KeyModifiers::NONE),
                    (KeyCode::Up, KeyModifiers::NONE),
                ],
            ),
            down: KeyBinding::new(
                "Move down",
                vec![
                    (KeyCode::Char('j'), KeyModifiers::NONE),
                    (KeyCode::Down, KeyModifiers::NONE),
                ],
            ),
            new_item: KeyBinding::new("New item", vec![(KeyCode::Char('n'), KeyModifiers::NONE)]),
            delete_item: KeyBinding::new(
                "Delete item",
                vec![(KeyCode::Char('d'), KeyModifiers::NONE)],
            ),
            toggle_section: KeyBinding::new(
                "Toggle sidebar section",
                vec![(KeyCode::Char('s'), KeyModifiers::NONE)],
            ),
            confirm: KeyBinding::new("Confirm", vec![(KeyCode::Enter, KeyModifiers::NONE)]),
            cancel: KeyBinding::new("Cancel / close", vec![(KeyCode::Esc, KeyModifiers::NONE)]),
            method_next: KeyBinding::new(
                "Next HTTP method",
                vec![
                    (KeyCode::Right, KeyModifiers::NONE),
                    (KeyCode::Char('l'), KeyModifiers::NONE),
                ],
            ),
            method_prev: KeyBinding::new(
                "Prev HTTP method",
                vec![
                    (KeyCode::Left, KeyModifiers::NONE),
                    (KeyCode::Char('h'), KeyModifiers::NONE),
                ],
            ),
            new_folder: KeyBinding::new(
                "New folder",
                vec![(KeyCode::Char('f'), KeyModifiers::NONE)],
            ),
            copy: KeyBinding::new(
                "Copy to clipboard",
                vec![(KeyCode::Char('y'), KeyModifiers::NONE)],
            ),
        }
    }

    pub fn get_bindings(&self) -> Vec<(&'static str, Vec<(KeyCode, KeyModifiers)>)> {
        vec![
            (self.quit.description, self.quit.keys.clone()),
            (self.help.description, self.help.keys.clone()),
            (self.send.description, self.send.keys.clone()),
            (self.save.description, self.save.keys.clone()),
            (self.tab_next.description, self.tab_next.keys.clone()),
            (self.tab_prev.description, self.tab_prev.keys.clone()),
            (self.up.description, self.up.keys.clone()),
            (self.down.description, self.down.keys.clone()),
            (self.new_item.description, self.new_item.keys.clone()),
            (self.delete_item.description, self.delete_item.keys.clone()),
            (
                self.toggle_section.description,
                self.toggle_section.keys.clone(),
            ),
            (self.method_next.description, self.method_next.keys.clone()),
            (self.method_prev.description, self.method_prev.keys.clone()),
            (self.new_folder.description, self.new_folder.keys.clone()),
            (self.copy.description, self.copy.keys.clone()),
        ]
    }
}
