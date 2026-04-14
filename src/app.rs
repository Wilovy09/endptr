use std::{path::PathBuf, sync::mpsc::Receiver};

use crossterm::event::KeyEvent;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    text::Span,
    widgets::{Clear, Paragraph, StatefulWidget, Widget},
};

use crate::{
    http_client,
    keybinds::KeyMap,
    models::{resolve_path, CurrentRequest, HttpResponse, ItemPath, PostmanCollection, Secrets},
    storage,
    widgets::{
        HelpModal, HelpModalState, Modal, RequestPanel, RequestPanelState, RequestTab,
        ResponsePanel, Sidebar, SidebarSection, SidebarState, TextInput, Toolbar, ToolbarFocus,
    },
};

// ── Focus ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Default, PartialEq, Clone)]
pub enum AppFocus {
    #[default]
    Sidebar,
    Method,
    Url,
    RequestPanel,
    Response,
}

impl AppFocus {
    pub fn next(&self) -> AppFocus {
        match self {
            AppFocus::Sidebar => AppFocus::Method,
            AppFocus::Method => AppFocus::Url,
            AppFocus::Url => AppFocus::RequestPanel,
            AppFocus::RequestPanel => AppFocus::Response,
            AppFocus::Response => AppFocus::Sidebar,
        }
    }

    pub fn prev(&self) -> AppFocus {
        match self {
            AppFocus::Sidebar => AppFocus::Response,
            AppFocus::Method => AppFocus::Sidebar,
            AppFocus::Url => AppFocus::Method,
            AppFocus::RequestPanel => AppFocus::Url,
            AppFocus::Response => AppFocus::RequestPanel,
        }
    }
}

// ── Modal context ─────────────────────────────────────────────────────────────

/// Tracks what a generic modal is being used for.
#[derive(Debug, Clone, PartialEq)]
pub enum ModalContext {
    SaveRequest,
    NewCollection,
    NewFolder,
    AddSecret,
    EditSecret { key: String },
    AddHeader,
    AddParam,
    DeleteRequest { col_idx: usize, item_path: ItemPath },
    DeleteCollection { col_idx: usize },
    DeleteSecret { key: String },
}

// ── App State ─────────────────────────────────────────────────────────────────

pub struct App {
    pub focus: AppFocus,

    /// All loaded collection files: (path, collection)
    pub collections: Vec<(PathBuf, PostmanCollection)>,
    pub secrets: Secrets,
    pub current: CurrentRequest,

    pub sidebar_state: SidebarState,
    pub request_panel_state: RequestPanelState,
    pub help_modal_state: HelpModalState,

    pub url_input: TextInput,
    pub body_input: TextInput,

    pub response: Option<Result<HttpResponse, String>>,
    pub response_scroll: u16,
    pub is_loading: bool,
    pub http_rx: Option<Receiver<Result<HttpResponse, String>>>,

    pub status: Option<String>,
    pub modal: Option<Modal>,
    pub modal_ctx: Option<ModalContext>,
}

impl Default for App {
    fn default() -> Self {
        let collections = storage::load_all_collections();
        let secrets = storage::load_secrets();

        let mut sidebar_state = SidebarState::default();
        sidebar_state.rebuild(&collections);

        Self {
            focus: AppFocus::default(),
            collections,
            secrets,
            current: CurrentRequest::default(),
            sidebar_state,
            request_panel_state: RequestPanelState::default(),
            help_modal_state: HelpModalState::default(),
            url_input: TextInput::default(),
            body_input: TextInput::default(),
            response: None,
            response_scroll: 0,
            is_loading: false,
            http_rx: None,
            status: None,
            modal: None,
            modal_ctx: None,
        }
    }
}

impl App {
    pub fn poll_http(&mut self) {
        if let Some(rx) = &self.http_rx {
            if let Ok(result) = rx.try_recv() {
                self.response = Some(result);
                self.is_loading = false;
                self.http_rx = None;
                self.focus = AppFocus::Response;
            }
        }
    }

    pub fn send_request(&mut self) {
        if self.is_loading {
            return;
        }
        // Build auth config from panel state
        let auth = crate::models::AuthConfig {
            auth_type: self.request_panel_state.auth_type.clone(),
            username: self.request_panel_state.auth_username.value.clone(),
            password: self.request_panel_state.auth_password.value.clone(),
            token: self.request_panel_state.auth_token.value.clone(),
        };
        let request = CurrentRequest {
            method: self.current.method.clone(),
            url: self.url_input.value.clone(),
            headers: self.current.headers.clone(),
            body: self.body_input.value.clone(),
            auth,
            params: self.current.params.clone(),
        };
        let secrets = self.secrets.clone();
        let (tx, rx) = std::sync::mpsc::channel();
        self.http_rx = Some(rx);
        self.is_loading = true;
        self.response = None;
        self.response_scroll = 0;
        std::thread::spawn(move || {
            let _ = tx.send(http_client::send_request(request, secrets));
        });
    }

    // ── Collection helpers ────────────────────────────────────────────────────

    fn rebuild_sidebar(&mut self) {
        self.sidebar_state.rebuild(&self.collections);
    }

    fn save_collection_at(&mut self, col_idx: usize) {
        if let Some((path, col)) = self.collections.get(col_idx) {
            match storage::save_collection(path, col) {
                Ok(_) => {}
                Err(e) => self.status = Some(format!("Save error: {e}")),
            }
        }
    }

    pub fn load_selected_request(&mut self) {
        let node = match self.sidebar_state.selected_node() {
            Some(n) if n.is_request() => n.clone(),
            _ => return,
        };

        let col_idx = node.col_idx;
        let item_path = node.item_path.clone();

        if let Some((_, col)) = self.collections.get(col_idx) {
            if let Some(item) = resolve_path(&col.item, &item_path) {
                if let Some(req) = &item.request {
                    let cr = CurrentRequest::from(req);
                    self.url_input = TextInput::new(cr.url.clone());
                    self.body_input = TextInput::new(cr.body.clone());
                    // Populate auth panel state from loaded config
                    self.request_panel_state.auth_type = cr.auth.auth_type.clone();
                    self.request_panel_state.auth_username = TextInput::new(cr.auth.username.clone());
                    self.request_panel_state.auth_password = TextInput::new(cr.auth.password.clone());
                    self.request_panel_state.auth_token = TextInput::new(cr.auth.token.clone());
                    self.request_panel_state.auth_field = 0;
                    self.current = cr;
                    self.response = None;
                    self.response_scroll = 0;
                    self.focus = AppFocus::Url;
                    self.status = Some(format!("Loaded: {}", item.name));
                }
            }
        }
    }

    pub fn save_request_as(&mut self, name: String) {
        // Save into the first collection by default (or the active one).
        // Future: let user pick target collection.
        let col_idx = self
            .sidebar_state
            .selected_node()
            .map(|n| n.col_idx)
            .unwrap_or(0);

        if let Some((_, col)) = self.collections.get_mut(col_idx) {
            let auth = crate::models::AuthConfig {
                auth_type: self.request_panel_state.auth_type.clone(),
                username: self.request_panel_state.auth_username.value.clone(),
                password: self.request_panel_state.auth_password.value.clone(),
                token: self.request_panel_state.auth_token.value.clone(),
            };
            let item = CurrentRequest {
                method: self.current.method.clone(),
                url: self.url_input.value.clone(),
                headers: self.current.headers.clone(),
                body: self.body_input.value.clone(),
                auth,
                params: self.current.params.clone(),
            }
            .to_postman_item(&name);

            if let Some(existing) = col.item.iter_mut().find(|i| i.name == name) {
                *existing = item;
            } else {
                col.item.push(item);
            }
        }
        self.save_collection_at(col_idx);
        self.rebuild_sidebar();
        self.status = Some(format!("Saved: {name}"));
    }

    pub fn add_folder(&mut self, name: String) {
        let col_idx = self
            .sidebar_state
            .selected_node()
            .map(|n| n.col_idx)
            .unwrap_or(0);

        if let Some((_, col)) = self.collections.get_mut(col_idx) {
            col.item.push(crate::models::PostmanItem::new_folder(&name));
        }
        self.save_collection_at(col_idx);
        self.rebuild_sidebar();
        self.status = Some(format!("Folder created: {name}"));
    }

    pub fn delete_selected(&mut self) {
        let node = match self.sidebar_state.selected_node() {
            Some(n) => n.clone(),
            None => return,
        };

        match node.kind {
            crate::widgets::FlatNodeKind::Collection => {
                if self.collections.len() <= 1 {
                    self.status = Some("Cannot delete last collection".to_string());
                    return;
                }
                let (path, _) = self.collections.remove(node.col_idx);
                let _ = storage::delete_collection(&path);
                self.status = Some(format!("Deleted: {}", node.label));
            }
            _ => {
                let col_idx = node.col_idx;
                let item_path = node.item_path.clone();

                if let Some((_, col)) = self.collections.get_mut(col_idx) {
                    if let Some((last_idx, parent_path)) = item_path.split_last() {
                        let parent = if parent_path.is_empty() {
                            Some(&mut col.item)
                        } else {
                            crate::models::resolve_path_mut(&mut col.item, parent_path)
                                .and_then(|p| p.item.as_mut())
                        };
                        if let Some(list) = parent {
                            list.remove(*last_idx);
                        }
                    }
                }
                self.save_collection_at(col_idx);
                self.status = Some(format!("Deleted: {}", node.label));
            }
        }
        self.rebuild_sidebar();
    }

    pub fn new_collection(&mut self, name: String) {
        match storage::create_collection(&name) {
            Ok((path, col)) => {
                self.collections.push((path, col));
                self.rebuild_sidebar();
                self.status = Some(format!("Collection created: {name}"));
            }
            Err(e) => self.status = Some(format!("Error: {e}")),
        }
    }

    pub fn add_secret(&mut self, key: String, val: String) {
        self.secrets.insert(key.clone(), val);
        let _ = storage::save_secrets(&self.secrets);
        self.status = Some(format!("Secret saved: {key}"));
    }

    pub fn delete_selected_secret(&mut self) {
        let mut keys: Vec<String> = self.secrets.keys().cloned().collect();
        keys.sort();
        if let Some(idx) = self.sidebar_state.secret_list.selected() {
            if let Some(key) = keys.get(idx) {
                let key = key.clone();
                self.secrets.remove(&key);
                let _ = storage::save_secrets(&self.secrets);
                self.status = Some(format!("Deleted secret: {key}"));
            }
        }
    }

    // ── Key handling ──────────────────────────────────────────────────────────

    pub fn handle_key(&mut self, key: KeyEvent, km: &KeyMap) -> bool {
        if km.quit.matches(&key) {
            return true;
        }

        if self.modal.is_some() {
            self.handle_modal_key(key, km);
            return false;
        }

        if km.help.matches(&key) {
            if self.help_modal_state.is_open {
                self.help_modal_state.close();
            } else {
                self.help_modal_state.open();
            }
            return false;
        }

        if self.help_modal_state.is_open {
            if km.cancel.matches(&key) {
                self.help_modal_state.close();
            }
            return false;
        }

        if km.send.matches(&key) {
            self.send_request();
            return false;
        }
        if km.save.matches(&key) {
            self.open_modal(Modal::SaveRequest {
                name_input: TextInput::default(),
            }, ModalContext::SaveRequest);
            return false;
        }
        if km.tab_next.matches(&key) {
            self.focus = self.focus.next();
            return false;
        }
        if km.tab_prev.matches(&key) {
            self.focus = self.focus.prev();
            return false;
        }

        match self.focus.clone() {
            AppFocus::Sidebar => self.handle_sidebar_key(key, km),
            AppFocus::Method => self.handle_method_key(key, km),
            AppFocus::Url => self.handle_url_key(key, km),
            AppFocus::RequestPanel => self.handle_request_panel_key(key, km),
            AppFocus::Response => self.handle_response_key(key, km),
        }
        false
    }

    fn open_modal(&mut self, modal: Modal, ctx: ModalContext) {
        self.modal = Some(modal);
        self.modal_ctx = Some(ctx);
    }

    fn handle_sidebar_key(&mut self, key: KeyEvent, km: &KeyMap) {
        match self.sidebar_state.section.clone() {
            SidebarSection::Requests => {
                if km.down.matches(&key) {
                    self.sidebar_state.next();
                } else if km.up.matches(&key) {
                    self.sidebar_state.prev();
                } else if km.confirm.matches(&key) {
                    // If request → load; if collection/folder → toggle
                    let is_req = self
                        .sidebar_state
                        .selected_node()
                        .map(|n| n.is_request())
                        .unwrap_or(false);
                    if is_req {
                        self.load_selected_request();
                    } else {
                        self.sidebar_state.toggle_selected();
                        self.rebuild_sidebar();
                    }
                } else if km.new_item.matches(&key) {
                    // 'n' opens submenu: new request or new folder or new collection
                    // For simplicity: if on collection level → new collection; else new request
                    let is_col = self
                        .sidebar_state
                        .selected_node()
                        .map(|n| n.is_collection())
                        .unwrap_or(true);
                    if is_col {
                        self.open_modal(
                            Modal::SaveRequest { name_input: TextInput::default() },
                            ModalContext::NewCollection,
                        );
                    } else {
                        self.open_modal(
                            Modal::SaveRequest { name_input: TextInput::default() },
                            ModalContext::SaveRequest,
                        );
                    }
                } else if km.new_folder.matches(&key) {
                    self.open_modal(
                        Modal::SaveRequest { name_input: TextInput::default() },
                        ModalContext::NewFolder,
                    );
                } else if km.delete_item.matches(&key) {
                    let label = self
                        .sidebar_state
                        .selected_node()
                        .map(|n| n.label.clone())
                        .unwrap_or_default();
                    if !label.is_empty() {
                        self.open_modal(
                            Modal::ConfirmDelete {
                                message: format!("Delete \"{label}\"?"),
                            },
                            ModalContext::SaveRequest, // dummy, delete_selected uses node
                        );
                    }
                } else if km.toggle_section.matches(&key) {
                    self.sidebar_state.toggle_section();
                }
            }
            SidebarSection::Secrets => {
                let max = self.secrets.len();
                if km.down.matches(&key) {
                    self.sidebar_state.next_secret(max);
                } else if km.up.matches(&key) {
                    self.sidebar_state.prev_secret(max);
                } else if km.new_item.matches(&key) {
                    self.open_modal(
                        Modal::AddSecret {
                            key_input: TextInput::default(),
                            val_input: TextInput::default(),
                            field: 0,
                        },
                        ModalContext::AddSecret,
                    );
                } else if km.delete_item.matches(&key) {
                    let mut keys: Vec<String> = self.secrets.keys().cloned().collect();
                    keys.sort();
                    if let Some(idx) = self.sidebar_state.secret_list.selected() {
                        if let Some(k) = keys.get(idx).cloned() {
                            self.open_modal(
                                Modal::ConfirmDelete {
                                    message: format!("Delete secret \"{k}\"?"),
                                },
                                ModalContext::DeleteSecret { key: k },
                            );
                        }
                    }
                } else if km.confirm.matches(&key) {
                    let mut keys: Vec<String> = self.secrets.keys().cloned().collect();
                    keys.sort();
                    if let Some(idx) = self.sidebar_state.secret_list.selected() {
                        if let Some(k) = keys.get(idx).cloned() {
                            let val = self.secrets.get(&k).cloned().unwrap_or_default();
                            self.open_modal(
                                Modal::EditSecret {
                                    key: k.clone(),
                                    val_input: TextInput::new(val),
                                },
                                ModalContext::EditSecret { key: k },
                            );
                        }
                    }
                } else if km.toggle_section.matches(&key) {
                    self.sidebar_state.toggle_section();
                }
            }
        }
    }

    fn handle_method_key(&mut self, key: KeyEvent, km: &KeyMap) {
        if km.method_next.matches(&key) {
            self.current.method = self.current.method.next();
        } else if km.method_prev.matches(&key) {
            self.current.method = self.current.method.prev();
        } else if km.confirm.matches(&key) {
            self.send_request();
        }
    }

    fn handle_url_key(&mut self, key: KeyEvent, km: &KeyMap) {
        use crossterm::event::KeyCode;
        if km.confirm.matches(&key) {
            self.send_request();
            return;
        }
        match key.code {
            KeyCode::Backspace => self.url_input.delete_before(),
            KeyCode::Delete => self.url_input.delete_after(),
            KeyCode::Left => self.url_input.move_left(),
            KeyCode::Right => self.url_input.move_right(),
            KeyCode::Home => self.url_input.move_to_start(),
            KeyCode::End => self.url_input.move_to_end(),
            KeyCode::Char(c) => self.url_input.insert(c),
            _ => {}
        }
    }

    fn handle_request_panel_key(&mut self, key: KeyEvent, km: &KeyMap) {
        use crossterm::event::KeyCode;

        // `[` / `]` cycle tabs regardless of current tab
        match key.code {
            KeyCode::Char('[') => {
                self.request_panel_state.tab = self.request_panel_state.tab.prev();
                return;
            }
            KeyCode::Char(']') => {
                self.request_panel_state.tab = self.request_panel_state.tab.next();
                return;
            }
            _ => {}
        }

        match self.request_panel_state.tab.clone() {
            RequestTab::Body => match key.code {
                KeyCode::Backspace => self.body_input.delete_before(),
                KeyCode::Delete => self.body_input.delete_after(),
                KeyCode::Left => self.body_input.move_left(),
                KeyCode::Right => self.body_input.move_right(),
                KeyCode::Home => self.body_input.move_to_start(),
                KeyCode::End => self.body_input.move_to_end(),
                KeyCode::Char('y') => {
                    match crate::highlight::copy_to_clipboard(&self.body_input.value) {
                        Ok(_) => self.status = Some("Body copied to clipboard".to_string()),
                        Err(e) => self.status = Some(format!("Copy failed: {e}")),
                    }
                }
                KeyCode::Char(c) => self.body_input.insert(c),
                _ => {}
            },

            RequestTab::Headers => {
                let max = self.current.headers.len();
                if km.down.matches(&key) {
                    let i = self
                        .request_panel_state
                        .header_list
                        .selected()
                        .map(|i| (i + 1) % max.max(1))
                        .unwrap_or(0);
                    self.request_panel_state.header_list.select(Some(i));
                } else if km.up.matches(&key) {
                    let i = self
                        .request_panel_state
                        .header_list
                        .selected()
                        .map(|i| if i == 0 { max.saturating_sub(1) } else { i - 1 })
                        .unwrap_or(0);
                    self.request_panel_state.header_list.select(Some(i));
                } else if km.new_item.matches(&key) {
                    self.open_modal(
                        Modal::AddSecret {
                            key_input: TextInput::default(),
                            val_input: TextInput::default(),
                            field: 0,
                        },
                        ModalContext::AddHeader,
                    );
                } else if km.delete_item.matches(&key) {
                    if let Some(idx) = self.request_panel_state.header_list.selected() {
                        if idx < self.current.headers.len() {
                            self.current.headers.remove(idx);
                        }
                    }
                }
            }

            RequestTab::Auth => match key.code {
                KeyCode::Left => {
                    self.request_panel_state.auth_type =
                        self.request_panel_state.auth_type.prev();
                    self.request_panel_state.auth_field = 0;
                }
                KeyCode::Right => {
                    self.request_panel_state.auth_type =
                        self.request_panel_state.auth_type.next();
                    self.request_panel_state.auth_field = 0;
                }
                KeyCode::Tab => self.request_panel_state.auth_next_field(),
                KeyCode::BackTab => self.request_panel_state.auth_prev_field(),
                KeyCode::Down => self.request_panel_state.auth_next_field(),
                KeyCode::Up => self.request_panel_state.auth_prev_field(),
                code => {
                    if let Some(input) = self.request_panel_state.active_auth_input() {
                        input_key(input, code);
                    }
                }
            },

            RequestTab::Params => {
                let max = self.current.params.len();
                if km.down.matches(&key) {
                    let i = self
                        .request_panel_state
                        .params_list
                        .selected()
                        .map(|i| (i + 1) % max.max(1))
                        .unwrap_or(0);
                    self.request_panel_state.params_list.select(Some(i));
                } else if km.up.matches(&key) {
                    let i = self
                        .request_panel_state
                        .params_list
                        .selected()
                        .map(|i| if i == 0 { max.saturating_sub(1) } else { i - 1 })
                        .unwrap_or(0);
                    self.request_panel_state.params_list.select(Some(i));
                } else if km.new_item.matches(&key) {
                    self.open_modal(
                        Modal::AddSecret {
                            key_input: TextInput::default(),
                            val_input: TextInput::default(),
                            field: 0,
                        },
                        ModalContext::AddParam,
                    );
                } else if km.delete_item.matches(&key) {
                    if let Some(idx) = self.request_panel_state.params_list.selected() {
                        if idx < self.current.params.len() {
                            self.current.params.remove(idx);
                        }
                    }
                } else if key.code == KeyCode::Char(' ') {
                    // Toggle enabled
                    if let Some(idx) = self.request_panel_state.params_list.selected() {
                        if let Some(p) = self.current.params.get_mut(idx) {
                            p.enabled = !p.enabled;
                        }
                    }
                }
            }
        }
    }

    fn handle_response_key(&mut self, key: KeyEvent, km: &KeyMap) {
        if km.down.matches(&key) {
            self.response_scroll = self.response_scroll.saturating_add(1);
        } else if km.up.matches(&key) {
            self.response_scroll = self.response_scroll.saturating_sub(1);
        } else if km.copy.matches(&key) {
            let text = match &self.response {
                Some(Ok(r)) => Some(r.body.clone()),
                _ => None,
            };
            if let Some(body) = text {
                match crate::highlight::copy_to_clipboard(&body) {
                    Ok(_) => self.status = Some("Response copied to clipboard".to_string()),
                    Err(e) => self.status = Some(format!("Copy failed: {e}")),
                }
            }
        }
    }

    fn handle_modal_key(&mut self, key: KeyEvent, km: &KeyMap) {
        use crossterm::event::KeyCode;

        let modal = match self.modal.take() {
            Some(m) => m,
            None => return,
        };
        let ctx = self.modal_ctx.take();

        match modal {
            Modal::SaveRequest { mut name_input } => {
                if km.confirm.matches(&key) {
                    let name = name_input.value.trim().to_string();
                    if !name.is_empty() {
                        match ctx {
                            Some(ModalContext::NewCollection) => self.new_collection(name),
                            Some(ModalContext::NewFolder) => self.add_folder(name),
                            _ => self.save_request_as(name),
                        }
                    }
                } else if km.cancel.matches(&key) {
                    // discard
                } else {
                    input_key(&mut name_input, key.code);
                    self.modal = Some(Modal::SaveRequest { name_input });
                    self.modal_ctx = ctx;
                }
            }

            Modal::AddSecret { mut key_input, mut val_input, mut field } => {
                if km.confirm.matches(&key) {
                    let k = key_input.value.trim().to_string();
                    let v = val_input.value.trim().to_string();
                    if !k.is_empty() {
                        if ctx == Some(ModalContext::AddHeader) {
                            self.current.headers.push((k, v));
                        } else if ctx == Some(ModalContext::AddParam) {
                            self.current.params.push(crate::models::QueryParam::new(k, v));
                        } else {
                            self.add_secret(k, v);
                        }
                    }
                } else if km.cancel.matches(&key) {
                } else if key.code == KeyCode::Tab {
                    field = if field == 0 { 1 } else { 0 };
                    self.modal = Some(Modal::AddSecret { key_input, val_input, field });
                    self.modal_ctx = ctx;
                } else {
                    let target = if field == 0 { &mut key_input } else { &mut val_input };
                    input_key(target, key.code);
                    self.modal = Some(Modal::AddSecret { key_input, val_input, field });
                    self.modal_ctx = ctx;
                }
            }

            Modal::EditSecret { key: secret_key, mut val_input } => {
                if km.confirm.matches(&key) {
                    let v = val_input.value.trim().to_string();
                    self.secrets.insert(secret_key.clone(), v);
                    let _ = storage::save_secrets(&self.secrets);
                    self.status = Some(format!("Updated: {secret_key}"));
                } else if km.cancel.matches(&key) {
                } else {
                    input_key(&mut val_input, key.code);
                    self.modal = Some(Modal::EditSecret { key: secret_key, val_input });
                    self.modal_ctx = ctx;
                }
            }

            Modal::ConfirmDelete { message } => {
                if key.code == KeyCode::Char('y') {
                    match ctx {
                        Some(ModalContext::DeleteSecret { key: k }) => {
                            self.secrets.remove(&k);
                            let _ = storage::save_secrets(&self.secrets);
                            self.status = Some(format!("Deleted secret: {k}"));
                        }
                        _ => {
                            self.delete_selected();
                        }
                    }
                }
                // any other key = cancel (modal already taken)
                let _ = message;
            }
        }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn input_key(input: &mut TextInput, code: crossterm::event::KeyCode) {
    use crossterm::event::KeyCode;
    match code {
        KeyCode::Backspace => input.delete_before(),
        KeyCode::Delete => input.delete_after(),
        KeyCode::Left => input.move_left(),
        KeyCode::Right => input.move_right(),
        KeyCode::Home => input.move_to_start(),
        KeyCode::End => input.move_to_end(),
        KeyCode::Char(c) => input.insert(c),
        _ => {}
    }
}

// ── AppUi ─────────────────────────────────────────────────────────────────────

pub struct AppUi<'a> {
    pub keymap: &'a KeyMap,
}

impl<'a> AppUi<'a> {
    pub fn new(keymap: &'a KeyMap) -> Self {
        Self { keymap }
    }
}

impl<'a> StatefulWidget for AppUi<'a> {
    type State = App;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        Clear.render(area, buf);

        let full_area = Rect {
            x: 0,
            y: 0,
            width: buf.area.width,
            height: buf.area.height,
        };

        let root =
            Layout::horizontal([Constraint::Percentage(20), Constraint::Percentage(80)])
                .split(area);

        // Sidebar
        Sidebar::new(&state.secrets, state.focus == AppFocus::Sidebar)
            .render(root[0], buf, &mut state.sidebar_state);

        // Right: toolbar | request panel | response
        let right = Layout::vertical([
            Constraint::Length(3),
            Constraint::Percentage(35),
            Constraint::Min(0),
        ])
        .split(root[1]);

        let toolbar_focus = match state.focus {
            AppFocus::Method => ToolbarFocus::Method,
            AppFocus::Url => ToolbarFocus::Url,
            _ => ToolbarFocus::None,
        };
        Toolbar::new(&state.current.method, &state.url_input, state.is_loading, toolbar_focus)
            .render(right[0], buf);

        RequestPanel::new(
            &state.body_input,
            &state.current.headers,
            &state.current.params,
            state.focus == AppFocus::RequestPanel,
        )
        .render(right[1], buf, &mut state.request_panel_state);

        ResponsePanel::new(
            state.response.as_ref(),
            state.is_loading,
            state.response_scroll,
            state.focus == AppFocus::Response,
        )
        .render(right[2], buf);

        // Status bar
        if let Some(ref msg) = state.status {
            let status_area = Rect {
                x: full_area.x,
                y: full_area.y + full_area.height.saturating_sub(1),
                width: full_area.width,
                height: 1,
            };
            Paragraph::new(Span::styled(
                format!(" {msg} "),
                Style::default().fg(Color::Black).bg(Color::Yellow),
            ))
            .render(status_area, buf);
        }

        HelpModal::new(self.keymap).render(full_area, buf, &mut state.help_modal_state);

        if let Some(ref modal) = state.modal {
            modal.render(full_area, buf);
        }
    }
}
