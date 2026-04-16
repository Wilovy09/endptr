use std::{collections::HashSet, path::PathBuf};

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Margin, Rect},
    style::{Color, Style},
    symbols::border::ROUNDED,
    text::{Line, Span},
    widgets::{Block, List, ListItem, ListState, Paragraph, StatefulWidget, Widget},
};

use crate::{
    models::{HttpMethod, ItemPath, OcCollection, OcItem, Secrets},
    workflow::Workflow,
};

// ── Section tabs ──────────────────────────────────────────────────────────────

#[derive(Debug, Default, PartialEq, Clone)]
pub enum SidebarSection {
    #[default]
    Requests,
    Secrets,
    Workflows,
}

// ── Tree flat node ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum FlatNodeKind {
    /// A loaded collection file
    Collection,
    /// A Postman folder within a collection
    Folder,
    /// A leaf request
    Request { method: String },
}

#[derive(Debug, Clone)]
pub struct FlatNode {
    pub kind: FlatNodeKind,
    pub depth: usize,
    pub label: String,
    /// Which collection (index into `collections` vec)
    pub col_idx: usize,
    /// Path within that collection's item tree. Empty = the collection itself.
    pub item_path: ItemPath,
    pub is_expanded: bool,
}

impl FlatNode {
    pub fn is_collection(&self) -> bool {
        self.kind == FlatNodeKind::Collection
    }
    pub fn is_folder(&self) -> bool {
        self.kind == FlatNodeKind::Folder
    }
    pub fn is_request(&self) -> bool {
        matches!(self.kind, FlatNodeKind::Request { .. })
    }
}

// ── Expand key ────────────────────────────────────────────────────────────────

/// Unique key for expand/collapse state: `"col_idx/0/2/1"`.
fn expand_key(col_idx: usize, item_path: &[usize]) -> String {
    let parts: Vec<String> = std::iter::once(col_idx.to_string())
        .chain(item_path.iter().map(|i| i.to_string()))
        .collect();
    parts.join("/")
}

// ── State ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Default)]
pub struct SidebarState {
    pub section: SidebarSection,
    pub expanded: HashSet<String>,
    pub list_state: ListState,
    pub secret_list: ListState,
    pub workflow_list: ListState,
    /// Cached flat tree — rebuilt when collections change or expand/collapse
    pub flat: Vec<FlatNode>,
}

impl SidebarState {
    /// Rebuild the flat list from current collections + expand state.
    pub fn rebuild(&mut self, collections: &[(PathBuf, OcCollection)]) {
        self.flat.clear();
        for (col_idx, (_, col)) in collections.iter().enumerate() {
            let col_key = expand_key(col_idx, &[]);
            let expanded = self.expanded.contains(&col_key);

            self.flat.push(FlatNode {
                kind: FlatNodeKind::Collection,
                depth: 0,
                label: col.name.clone(),
                col_idx,
                item_path: vec![],
                is_expanded: expanded,
            });

            if expanded {
                push_items(&mut self.flat, &col.items, col_idx, &[], 1, &self.expanded);
            }
        }
    }

    pub fn toggle_selected(&mut self) {
        if let Some(idx) = self.list_state.selected() {
            if let Some(node) = self.flat.get(idx).cloned() {
                if node.is_request() {
                    return; // requests can't expand
                }
                let key = expand_key(node.col_idx, &node.item_path);
                if self.expanded.contains(&key) {
                    self.expanded.remove(&key);
                } else {
                    self.expanded.insert(key);
                }
            }
        }
    }

    /// Call after toggle to refresh flat list.
    pub fn selected_node(&self) -> Option<&FlatNode> {
        self.flat.get(self.list_state.selected()?)
    }

    pub fn next(&mut self) {
        let max = self.flat.len();
        if max == 0 {
            return;
        }
        let i = self
            .list_state
            .selected()
            .map(|i| (i + 1) % max)
            .unwrap_or(0);
        self.list_state.select(Some(i));
    }

    pub fn prev(&mut self) {
        let max = self.flat.len();
        if max == 0 {
            return;
        }
        let i = self
            .list_state
            .selected()
            .map(|i| if i == 0 { max - 1 } else { i - 1 })
            .unwrap_or(0);
        self.list_state.select(Some(i));
    }

    pub fn next_secret(&mut self, max: usize) {
        if max == 0 {
            return;
        }
        let i = self
            .secret_list
            .selected()
            .map(|i| (i + 1) % max)
            .unwrap_or(0);
        self.secret_list.select(Some(i));
    }

    pub fn prev_secret(&mut self, max: usize) {
        if max == 0 {
            return;
        }
        let i = self
            .secret_list
            .selected()
            .map(|i| if i == 0 { max - 1 } else { i - 1 })
            .unwrap_or(0);
        self.secret_list.select(Some(i));
    }

    pub fn next_workflow(&mut self, max: usize) {
        if max == 0 {
            return;
        }
        let i = self
            .workflow_list
            .selected()
            .map(|i| (i + 1) % max)
            .unwrap_or(0);
        self.workflow_list.select(Some(i));
    }

    pub fn prev_workflow(&mut self, max: usize) {
        if max == 0 {
            return;
        }
        let i = self
            .workflow_list
            .selected()
            .map(|i| if i == 0 { max - 1 } else { i - 1 })
            .unwrap_or(0);
        self.workflow_list.select(Some(i));
    }

    pub fn toggle_section(&mut self) {
        self.section = match self.section {
            SidebarSection::Requests => SidebarSection::Secrets,
            SidebarSection::Secrets => SidebarSection::Workflows,
            SidebarSection::Workflows => SidebarSection::Requests,
        };
    }
}

/// Recursively push items into the flat list.
fn push_items(
    flat: &mut Vec<FlatNode>,
    items: &[OcItem],
    col_idx: usize,
    parent_path: &[usize],
    depth: usize,
    expanded: &HashSet<String>,
) {
    for (i, item) in items.iter().enumerate() {
        let mut path = parent_path.to_vec();
        path.push(i);
        let key = expand_key(col_idx, &path);
        let is_expanded = expanded.contains(&key);

        match item {
            OcItem::Folder(folder) => {
                flat.push(FlatNode {
                    kind: FlatNodeKind::Folder,
                    depth,
                    label: folder.name.clone(),
                    col_idx,
                    item_path: path.clone(),
                    is_expanded,
                });
                if is_expanded {
                    push_items(flat, &folder.items, col_idx, &path, depth + 1, expanded);
                }
            }
            OcItem::Request(req) => {
                flat.push(FlatNode {
                    kind: FlatNodeKind::Request {
                        method: req.http.method.clone(),
                    },
                    depth,
                    label: req.info.name.clone(),
                    col_idx,
                    item_path: path,
                    is_expanded: false,
                });
            }
        }
    }
}

// ── Widget ────────────────────────────────────────────────────────────────────

pub struct Sidebar<'a> {
    pub secrets: &'a Secrets,
    pub workflows: &'a [(PathBuf, Workflow)],
    pub focused: bool,
}

impl<'a> Sidebar<'a> {
    pub fn new(
        secrets: &'a Secrets,
        workflows: &'a [(PathBuf, Workflow)],
        focused: bool,
    ) -> Self {
        Self {
            secrets,
            workflows,
            focused,
        }
    }
}

impl<'a> StatefulWidget for Sidebar<'a> {
    type State = SidebarState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let layout = Layout::vertical([Constraint::Length(3), Constraint::Min(0)]).split(area);

        // ── Tab bar ───────────────────────────────────────────────────────────
        {
            let tabs = Layout::horizontal([
                Constraint::Percentage(34),
                Constraint::Percentage(33),
                Constraint::Percentage(33),
            ])
            .split(layout[0].inner(Margin::new(1, 1)));

            Block::bordered().border_set(ROUNDED).render(layout[0], buf);

            let tab_style = |active: bool| {
                if active {
                    Style::new().fg(Color::Yellow).bold()
                } else {
                    Style::new().fg(Color::DarkGray)
                }
            };

            Paragraph::new("Requests")
                .centered()
                .style(tab_style(state.section == SidebarSection::Requests))
                .render(tabs[0], buf);
            Paragraph::new("Secrets")
                .centered()
                .style(tab_style(state.section == SidebarSection::Secrets))
                .render(tabs[1], buf);
            Paragraph::new("Workflows")
                .centered()
                .style(tab_style(state.section == SidebarSection::Workflows))
                .render(tabs[2], buf);
        }

        // ── Content ───────────────────────────────────────────────────────────
        let focus_color = if self.focused {
            Color::Yellow
        } else {
            Color::White
        };
        let block = Block::bordered()
            .border_set(ROUNDED)
            .border_style(Style::new().fg(focus_color));

        match state.section {
            SidebarSection::Requests => {
                let items: Vec<ListItem> = state
                    .flat
                    .iter()
                    .map(|node| {
                        let indent = "  ".repeat(node.depth);
                        let line = match &node.kind {
                            FlatNodeKind::Collection => {
                                let arrow = if node.is_expanded { "▼ " } else { "▶ " };
                                Line::from(vec![
                                    Span::raw(indent),
                                    Span::styled(
                                        format!("{arrow}{}", node.label),
                                        Style::new().fg(Color::White).bold(),
                                    ),
                                ])
                            }
                            FlatNodeKind::Folder => {
                                let arrow = if node.is_expanded { "▼ " } else { "▶ " };
                                Line::from(vec![
                                    Span::raw(indent),
                                    Span::styled(
                                        format!("{arrow}{}", node.label),
                                        Style::new().fg(Color::LightBlue),
                                    ),
                                ])
                            }
                            FlatNodeKind::Request { method } => {
                                let color = HttpMethod::from_str(method).color();
                                Line::from(vec![
                                    Span::raw(indent),
                                    Span::styled(
                                        format!("{:<7}", method),
                                        Style::new().fg(color).bold(),
                                    ),
                                    Span::raw(node.label.clone()),
                                ])
                            }
                        };
                        ListItem::new(line)
                    })
                    .collect();

                let list = List::new(items)
                    .block(block)
                    .highlight_style(Style::new().bg(Color::DarkGray));

                StatefulWidget::render(list, layout[1], buf, &mut state.list_state);
            }

            SidebarSection::Secrets => {
                let mut pairs: Vec<(&String, &String)> = self.secrets.iter().collect();
                pairs.sort_by_key(|(k, _)| k.as_str());

                let items: Vec<ListItem> = pairs
                    .iter()
                    .map(|(k, v)| {
                        ListItem::new(Line::from(vec![
                            Span::styled(k.as_str(), Style::new().fg(Color::Cyan)),
                            Span::styled(
                                format!(" = {}", &v[..v.len().min(20)]),
                                Style::new().fg(Color::DarkGray),
                            ),
                        ]))
                    })
                    .collect();

                let list = List::new(items)
                    .block(block)
                    .highlight_style(Style::new().bg(Color::DarkGray));

                StatefulWidget::render(list, layout[1], buf, &mut state.secret_list);
            }

            SidebarSection::Workflows => {
                let items: Vec<ListItem> = self
                    .workflows
                    .iter()
                    .map(|(_, wf)| {
                        ListItem::new(Line::from(vec![
                            Span::styled("▶ ", Style::new().fg(Color::Magenta)),
                            Span::styled(wf.name.clone(), Style::new().fg(Color::White)),
                            Span::styled(
                                format!("  ({} steps)", wf.steps.len()),
                                Style::new().fg(Color::DarkGray),
                            ),
                        ]))
                    })
                    .collect();

                let list = List::new(items)
                    .block(block)
                    .highlight_style(Style::new().bg(Color::DarkGray));

                StatefulWidget::render(list, layout[1], buf, &mut state.workflow_list);
            }
        }
    }
}
