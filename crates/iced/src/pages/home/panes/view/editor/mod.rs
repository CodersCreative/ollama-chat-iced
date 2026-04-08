use crate::{
    Application, InputMessage, Message, font::BODY_SIZE,
    pages::home::panes::view::HomePaneViewMessage, style,
};
use iced::{
    Element, Length, Padding,
    alignment::Vertical,
    widget::{
        Id as WidgetId, button, column, container, hover, right, row, scrollable, space, svg,
        text_input,
    },
    window,
};
use iced_code_editor::{
    CodeEditor, LspDocument, LspEvent, LspProcessClient, lsp_language_for_path,
};
use iced_drop::droppable;
use iced_selection::text;
use iced_split::{Strategy, horizontal_split, vertical_split};
use iced_term::{Terminal, TerminalView};
use std::{
    collections::HashMap,
    fs,
    path::Path,
    sync::{
        atomic::{AtomicU64, Ordering},
        mpsc,
    },
};

pub mod message;
pub use message::EditorViewMessage;

static TERMINAL_ID_COUNTER: AtomicU64 = AtomicU64::new(1);
const EDITOR_SIDEBAR_DEFAULT: f32 = 280.0;
const EDITOR_SIDEBAR_MIN: f32 = 190.0;
const EDITOR_TERMINAL_DEFAULT: f32 = 200.0;
const EDITOR_TERMINAL_MIN: f32 = 150.0;

#[derive(Debug, Clone)]
pub struct FileTreeNode {
    pub path: String,
    pub is_dir: bool,
    pub children: Vec<Self>,
}

pub struct EditorTab {
    pub title: String,
    pub path: Option<String>,
    pub editor: CodeEditor,
}

pub struct EditorTerminal {
    pub id: u64,
    pub title: String,
    pub terminal: Terminal,
}

impl std::fmt::Debug for EditorTerminal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EditorTerminal")
            .field("id", &self.id)
            .field("title", &self.title)
            .finish()
    }
}

impl Clone for EditorTerminal {
    fn clone(&self) -> Self {
        EditorView::new_terminal(None).unwrap_or_else(|_| Self {
            id: self.id,
            title: self.title.clone(),
            terminal: Terminal::new(self.id, iced_term::settings::Settings::default())
                .expect("failed to clone terminal"),
        })
    }
}

impl std::fmt::Debug for EditorTab {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EditorTab")
            .field("title", &self.title)
            .field("path", &self.path)
            .finish()
    }
}

impl Clone for EditorTab {
    fn clone(&self) -> Self {
        let syntax = self
            .path
            .as_deref()
            .map(EditorView::syntax_from_path)
            .unwrap_or("rs");

        let mut editor = EditorView::build_editor(
            &self.editor.content(),
            syntax,
            &iced::Theme::TokyoNightStorm,
        );
        if self.editor.is_modified() {
            let _ = editor.reset(&self.editor.content());
        } else {
            editor.mark_saved();
        }

        Self {
            title: self.title.clone(),
            path: self.path.clone(),
            editor,
        }
    }
}

pub struct EditorView {
    pub window_id: Option<window::Id>,
    pub folder_path: Option<String>,
    pub tree: Vec<FileTreeNode>,
    pub collapsed: Vec<String>,
    pub editing: HashMap<String, String>,
    pub dragging: Option<String>,
    pub tabs: Vec<EditorTab>,
    pub active_tab: usize,
    pub terminals: Vec<EditorTerminal>,
    pub active_terminal: usize,
    pub terminal_panel_open: bool,
    pub sidebar_split: f32,
    pub terminal_split: f32,
}

impl std::fmt::Debug for EditorView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EditorView")
            .field("window_id", &self.window_id)
            .field("folder_path", &self.folder_path)
            .field("tree", &self.tree)
            .field("tabs_len", &self.tabs.len())
            .field("active_tab", &self.active_tab)
            .finish()
    }
}

impl Clone for EditorView {
    fn clone(&self) -> Self {
        Self {
            window_id: self.window_id,
            folder_path: self.folder_path.clone(),
            tree: self.tree.clone(),
            collapsed: self.collapsed.clone(),
            editing: self.editing.clone(),
            dragging: self.dragging.clone(),
            tabs: self.tabs.clone(),
            active_tab: self.active_tab,
            terminals: self.terminals.clone(),
            active_terminal: self.active_terminal,
            terminal_panel_open: self.terminal_panel_open,
            sidebar_split: self.sidebar_split,
            terminal_split: self.terminal_split,
        }
    }
}

impl EditorView {
    pub fn new(theme: &iced::Theme) -> Self {
        Self {
            window_id: None,
            folder_path: None,
            tree: Vec::new(),
            collapsed: Vec::new(),
            editing: HashMap::new(),
            dragging: None,
            tabs: vec![Self::scratch_tab(theme)],
            active_tab: 0,
            terminals: vec![Self::new_terminal(None).expect("failed to create terminal")],
            active_terminal: 0,
            terminal_panel_open: false,
            sidebar_split: EDITOR_SIDEBAR_DEFAULT,
            terminal_split: EDITOR_TERMINAL_DEFAULT,
        }
    }

    fn scratch_tab(theme: &iced::Theme) -> EditorTab {
        EditorTab {
            title: "scratch.rs".to_string(),
            path: None,
            editor: Self::build_editor(Self::starter_content(), "rs", theme),
        }
    }

    fn next_terminal_id() -> u64 {
        TERMINAL_ID_COUNTER.fetch_add(1, Ordering::Relaxed)
    }

    pub fn new_terminal(working_directory: Option<String>) -> Result<EditorTerminal, String> {
        let mut settings = iced_term::settings::Settings::default();

        settings.backend.program =
            std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string());
        settings.backend.working_directory = working_directory.map(Into::into);

        let id = Self::next_terminal_id();
        let terminal = Terminal::new(id, settings).map_err(|e| e.to_string())?;

        Ok(EditorTerminal {
            id,
            title: format!("terminal {}", id),
            terminal,
        })
    }

    fn build_editor(content: &str, syntax: &str, theme: &iced::Theme) -> CodeEditor {
        let mut editor = CodeEditor::new(content, syntax)
            .with_wrap_enabled(true)
            .with_line_numbers_enabled(true)
            .with_viewport_height(700.0);
        editor.set_theme(iced_code_editor::from_iced_theme(theme));
        editor.set_font_size(14.0, true);
        editor.set_lsp_enabled(true);
        editor.mark_saved();
        editor
    }

    fn file_uri(path: &Path) -> Option<String> {
        let absolute = if path.is_absolute() {
            path.to_path_buf()
        } else {
            std::fs::canonicalize(path).ok()?
        };

        Some(format!("file://{}", absolute.to_string_lossy()))
    }

    fn lsp_root_uri(folder_path: Option<&str>, path: &Path) -> Option<String> {
        let root = folder_path
            .map(Path::new)
            .filter(|root| path.starts_with(root))
            .unwrap_or_else(|| path.parent().unwrap_or(path));
        Self::file_uri(root)
    }

    fn attach_lsp_for_path(
        folder_path: Option<&str>,
        editor: &mut CodeEditor,
        path: &str,
    ) -> Result<(), String> {
        let path_ref = Path::new(path);
        let Some(language) = lsp_language_for_path(path_ref) else {
            editor.detach_lsp();
            return Ok(());
        };

        let Some(root_uri) = Self::lsp_root_uri(folder_path, path_ref) else {
            return Ok(());
        };
        let Some(document_uri) = Self::file_uri(path_ref) else {
            return Ok(());
        };

        let (events, _receiver) = mpsc::channel::<LspEvent>();
        let client = LspProcessClient::new_with_server(&root_uri, events, language.server_key)?;
        editor.attach_lsp(
            Box::new(client),
            LspDocument::new(&document_uri, language.language_id),
        );
        Ok(())
    }

    pub fn sync_tab_lsp_for_folder(
        folder_path: Option<&str>,
        tab: &mut EditorTab,
    ) -> Result<(), String> {
        if let Some(path) = tab.path.as_deref() {
            Self::attach_lsp_for_path(folder_path, &mut tab.editor, path)
        } else {
            tab.editor.detach_lsp();
            Ok(())
        }
    }

    fn notify_active_tab_saved(&mut self) {
        self.active_tab_mut().editor.lsp_did_save();
    }

    pub fn snap_sidebar_split(split: f32) -> f32 {
        if (EDITOR_SIDEBAR_DEFAULT - 25.0..=EDITOR_SIDEBAR_DEFAULT + 25.0).contains(&split) {
            EDITOR_SIDEBAR_DEFAULT
        } else {
            split.max(EDITOR_SIDEBAR_MIN)
        }
    }

    pub fn snap_terminal_split(split: f32) -> f32 {
        if (EDITOR_TERMINAL_DEFAULT - 25.0..=EDITOR_TERMINAL_DEFAULT + 25.0).contains(&split) {
            EDITOR_TERMINAL_DEFAULT
        } else {
            split.max(EDITOR_TERMINAL_MIN)
        }
    }

    fn starter_content() -> &'static str {
        r#"fn main() {
    println!("Welcome to ochat!");
}

// Open a folder, work in tabs, and split an AI chat beside the editor when you want help.
"#
    }

    fn syntax_from_path(path: &str) -> &'static str {
        match Path::new(path)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or_default()
            .to_ascii_lowercase()
            .as_str()
        {
            "rs" => "rs",
            "py" => "py",
            "js" => "javascript",
            "jsx" => "jsx",
            "ts" => "typescript",
            "tsx" => "tsx",
            "json" => "json",
            "toml" => "toml",
            "yaml" | "yml" => "yaml",
            "md" => "markdown",
            "html" => "html",
            "css" => "css",
            "sh" => "bash",
            "go" => "go",
            "java" => "java",
            "c" => "c",
            "h" => "c",
            "cpp" | "cc" | "cxx" | "hpp" => "cpp",
            _ => "txt",
        }
    }

    fn active_tab(&self) -> &EditorTab {
        self.tabs
            .get(self.active_tab)
            .or_else(|| self.tabs.first())
            .unwrap()
    }

    fn active_tab_mut(&mut self) -> &mut EditorTab {
        let index = self.active_tab.min(self.tabs.len().saturating_sub(1));
        &mut self.tabs[index]
    }

    fn title_from_path(path: &str) -> String {
        Path::new(path)
            .file_name()
            .and_then(|name| name.to_str())
            .map(|name| name.to_string())
            .unwrap_or_else(|| path.to_string())
    }

    pub fn terminal_subscriptions<'a>(
        &'a self,
        editor_id: u32,
    ) -> Vec<iced::Subscription<Message>> {
        self.terminals
            .iter()
            .map(|term| {
                term.terminal
                    .subscription()
                    .with(editor_id)
                    .map(move |(x, y)| {
                        Message::HomePaneView(HomePaneViewMessage::Editor(
                            x,
                            EditorViewMessage::TerminalEvent(y),
                        ))
                    })
            })
            .collect()
    }

    async fn pick_file() -> Result<(String, String), String> {
        let Some(file) = rfd::AsyncFileDialog::new().pick_file().await else {
            return Err("Failed".to_string());
        };

        let path = file
            .path()
            .to_path_buf()
            .into_os_string()
            .into_string()
            .map_err(|_| "Failed to read selected path".to_string())?;

        let content = tokio::fs::read_to_string(&path)
            .await
            .map_err(|e| e.to_string())?;

        Ok((path, content))
    }

    async fn pick_folder() -> Result<(String, Vec<FileTreeNode>), String> {
        let Some(folder) = rfd::AsyncFileDialog::new().pick_folder().await else {
            return Err("Failed".to_string());
        };

        let root = folder
            .path()
            .to_path_buf()
            .into_os_string()
            .into_string()
            .map_err(|_| "Failed to read selected folder".to_string())?;

        Ok((root.clone(), Self::read_folder_entries(&root)?))
    }

    fn read_folder_entries(dir: &str) -> Result<Vec<FileTreeNode>, String> {
        let mut read = fs::read_dir(dir)
            .map_err(|e| format!("Failed to read folder '{}': {}", dir, e))?
            .flatten()
            .collect::<Vec<_>>();

        read.sort_by_key(|entry| (!entry.path().is_dir(), entry.path()));

        let mut entries = Vec::new();

        for entry in read {
            let path = entry.path();
            let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
                continue;
            };

            if name.starts_with('.') && path.is_dir() {
                continue;
            }

            let path_str = path.to_string_lossy().to_string();
            let is_dir = path.is_dir();

            entries.push(FileTreeNode {
                path: path_str.clone(),
                is_dir,
                children: if is_dir {
                    Self::read_folder_entries(&path_str)?
                } else {
                    Vec::new()
                },
            });
        }

        Ok(entries)
    }

    async fn save_file(path: String, content: String) -> Result<String, String> {
        tokio::fs::write(&path, content)
            .await
            .map_err(|e| e.to_string())?;
        Ok(path)
    }

    async fn pick_save_path(content: String, current_name: String) -> Result<String, String> {
        let Some(file) = rfd::AsyncFileDialog::new()
            .set_file_name(&current_name)
            .save_file()
            .await
        else {
            return Err("Failed".to_string());
        };

        let path = file
            .path()
            .to_path_buf()
            .into_os_string()
            .into_string()
            .map_err(|_| "Failed to save to selected path".to_string())?;

        Self::save_file(path, content).await
    }

    fn unique_child_path(dir: &str, base_name: &str, is_dir: bool) -> String {
        let base = Path::new(dir).join(base_name);
        if !base.exists() {
            return base.to_string_lossy().to_string();
        }

        let stem = Path::new(base_name)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(base_name);
        let ext = Path::new(base_name)
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or_default();

        for index in 2..1000 {
            let candidate = if is_dir || ext.is_empty() {
                Path::new(dir).join(format!("{} {}", stem, index))
            } else {
                Path::new(dir).join(format!("{} {}.{}", stem, index, ext))
            };

            if !candidate.exists() {
                return candidate.to_string_lossy().to_string();
            }
        }

        base.to_string_lossy().to_string()
    }

    async fn create_path(parent: String, is_dir: bool) -> Result<String, String> {
        let path = if is_dir {
            Self::unique_child_path(&parent, "new-folder", true)
        } else {
            Self::unique_child_path(&parent, "new_file.rs", false)
        };

        if is_dir {
            tokio::fs::create_dir_all(&path)
                .await
                .map_err(|e| e.to_string())?;
        } else {
            tokio::fs::write(&path, "")
                .await
                .map_err(|e| e.to_string())?;
        }

        Ok(path)
    }

    fn entry_name(path: &str) -> String {
        Path::new(path)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or(path)
            .to_string()
    }

    fn refresh_tree(&mut self) -> Result<(), String> {
        if let Some(root) = self.folder_path.clone() {
            self.tree = Self::read_folder_entries(&root)?;
        }

        Ok(())
    }

    fn close_tabs_for_path(&mut self, path: &str) {
        self.tabs.retain(|tab| {
            tab.path
                .as_ref()
                .map(|tab_path| {
                    !(tab_path == path
                        || tab_path
                            .starts_with(&(path.to_string() + std::path::MAIN_SEPARATOR_STR)))
                })
                .unwrap_or(true)
        });

        if self.tabs.is_empty() {
            self.tabs
                .push(Self::scratch_tab(&iced::Theme::TokyoNightStorm));
        }

        self.active_tab = self.active_tab.min(self.tabs.len().saturating_sub(1));
    }

    fn rename_tab_paths(&mut self, from: &str, to: &str) {
        for tab in &mut self.tabs {
            if let Some(path) = &mut tab.path {
                if path == from {
                    *path = to.to_string();
                    tab.title = Self::title_from_path(to);
                } else if path.starts_with(&(from.to_string() + std::path::MAIN_SEPARATOR_STR)) {
                    *path = path.replacen(from, to, 1);
                    tab.title = Self::title_from_path(path);
                }
            }
        }

        let mut tabs = std::mem::take(&mut self.tabs);
        for tab in &mut tabs {
            let _ = Self::sync_tab_lsp_for_folder(self.folder_path.as_deref(), tab);
        }
        self.tabs = tabs;
    }

    fn view_tree_node<'a>(
        &'a self,
        id: u32,
        node: &'a FileTreeNode,
        depth: usize,
    ) -> Element<'a, Message> {
        let path = node.path.clone();
        let is_collapsed = self.collapsed.contains(&node.path);
        let is_editing = self.editing.get(&node.path);
        let is_dragging = self.dragging.as_deref() == Some(node.path.as_str());
        let indent = space::horizontal().width((depth as f32 * 14.0).max(0.0));
        let icon = if node.is_dir {
            if is_collapsed {
                "arrow_drop_down.svg"
            } else {
                "arrow_drop_up.svg"
            }
        } else {
            "code.svg"
        };

        let name: Element<'a, Message> = if let Some(current) = is_editing {
            text_input("Rename...", current)
                .on_input({
                    let path = path.clone();
                    move |value| {
                        Message::HomePaneView(HomePaneViewMessage::Editor(
                            id,
                            EditorViewMessage::RenameInput(
                                path.clone(),
                                InputMessage::Update(value),
                            ),
                        ))
                    }
                })
                .on_submit(Message::HomePaneView(HomePaneViewMessage::Editor(
                    id,
                    EditorViewMessage::RenameSubmit(path.clone()),
                )))
                .style(style::text_input::input)
                .into()
        } else {
            text(Self::entry_name(&node.path))
                .size(BODY_SIZE)
                .style(if node.is_dir {
                    style::text::primary
                } else {
                    style::text::text
                })
                .into()
        };

        let main = row![
            indent,
            svg(svg::Handle::from_path(crate::utils::get_path_assets(
                icon.to_string()
            )))
            .style(style::svg::text)
            .width(BODY_SIZE + 4),
            name
        ]
        .spacing(6)
        .align_y(Vertical::Center)
        .width(Length::Fill);

        let controls = if is_editing.is_some() {
            row![
                style::svg_button::text("send.svg", BODY_SIZE).on_press(Message::HomePaneView(
                    HomePaneViewMessage::Editor(id, EditorViewMessage::RenameSubmit(path.clone()))
                )),
            ]
        } else {
            let mut controls: Vec<Element<'a, Message>> = if node.is_dir {
                vec![
                    style::svg_button::text("add.svg", BODY_SIZE)
                        .on_press(Message::HomePaneView(HomePaneViewMessage::Editor(
                            id,
                            EditorViewMessage::CreateFileIn(path.clone()),
                        )))
                        .into(),
                    style::svg_button::text("folder_new.svg", BODY_SIZE)
                        .on_press(Message::HomePaneView(HomePaneViewMessage::Editor(
                            id,
                            EditorViewMessage::CreateFolderIn(path.clone()),
                        )))
                        .into(),
                ]
            } else {
                Vec::new()
            };

            controls.append(&mut vec![
                style::svg_button::text("edit.svg", BODY_SIZE)
                    .on_press(Message::HomePaneView(HomePaneViewMessage::Editor(
                        id,
                        EditorViewMessage::RenameStart(path.clone()),
                    )))
                    .into(),
                style::svg_button::danger("delete.svg", BODY_SIZE)
                    .on_press(Message::HomePaneView(HomePaneViewMessage::Editor(
                        id,
                        EditorViewMessage::DeletePath(path.clone()),
                    )))
                    .into(),
            ]);

            row(controls).spacing(4)
        };

        let press = if node.is_dir {
            Message::HomePaneView(HomePaneViewMessage::Editor(
                id,
                EditorViewMessage::ToggleFolder(path.clone()),
            ))
        } else {
            Message::HomePaneView(HomePaneViewMessage::Editor(
                id,
                EditorViewMessage::OpenEntry(path.clone()),
            ))
        };

        let row_el = droppable(
            container(hover(
                button(main)
                    .style(style::button::transparent_back_white_text)
                    .padding(Padding::from([4.0, 8.0]))
                    .width(Length::Fill)
                    .on_press(press),
                right(container(controls).padding(Padding::from([0.0, 4.0]))),
            ))
            .width(Length::Fill)
            .style(if self.dragging.is_some() && node.is_dir && !is_dragging {
                style::container::drop_target
            } else if is_dragging {
                style::container::chat_back
            } else {
                iced::widget::container::transparent
            })
            .id(Self::tree_node_widget_id(id, &node.path, node.is_dir)),
        )
        .on_drag({
            let path = path.clone();
            move |point, _| {
                Message::HomePaneView(HomePaneViewMessage::Editor(
                    id,
                    EditorViewMessage::DragMove(path.clone(), point),
                ))
            }
        })
        .on_drop({
            let path = path.clone();
            move |point, _| {
                Message::HomePaneView(HomePaneViewMessage::Editor(
                    id,
                    EditorViewMessage::Drop(path.clone(), point),
                ))
            }
        })
        .on_cancel(Message::HomePaneView(HomePaneViewMessage::Editor(
            id,
            EditorViewMessage::CancelDrag,
        )))
        .drag_threshold(0.0)
        .drag_center(true)
        .drag_overlay(true);

        let mut body = column![row_el].spacing(2);

        if node.is_dir && !is_collapsed && !node.children.is_empty() {
            for child in &node.children {
                body = body.push(self.view_tree_node(id, child, depth + 1));
            }
        }

        body.into()
    }

    pub fn view_toolbar<'a>(
        &'a self,
        id: u32,
        _pane: iced::widget::pane_grid::Pane,
    ) -> Element<'a, Message> {
        let size = 16;

        let mut toolbar: Vec<Element<'a, Message>> = vec![
            style::svg_button::text("folder_open.svg", size)
                .on_press(Message::HomePaneView(HomePaneViewMessage::Editor(
                    id,
                    EditorViewMessage::OpenFolder,
                )))
                .into(),
            style::svg_button::text("file_open.svg", size)
                .on_press(Message::HomePaneView(HomePaneViewMessage::Editor(
                    id,
                    EditorViewMessage::OpenFile,
                )))
                .into(),
        ];

        if self.folder_path.is_some() {
            toolbar.append(&mut vec![
                style::svg_button::text("folder_new.svg", size)
                    .on_press(Message::HomePaneView(HomePaneViewMessage::Editor(
                        id,
                        EditorViewMessage::CreateFolderRoot,
                    )))
                    .into(),
                style::svg_button::text("file_new.svg", size)
                    .on_press(Message::HomePaneView(HomePaneViewMessage::Editor(
                        id,
                        EditorViewMessage::CreateFileRoot,
                    )))
                    .into(),
            ]);
        }

        toolbar.append(&mut vec![
            space::horizontal().into(),
            style::svg_button::text("save.svg", size)
                .on_press(Message::HomePaneView(HomePaneViewMessage::Editor(
                    id,
                    EditorViewMessage::Save,
                )))
                .into(),
            style::svg_button::text("save_as.svg", size)
                .on_press(Message::HomePaneView(HomePaneViewMessage::Editor(
                    id,
                    EditorViewMessage::SaveAs,
                )))
                .into(),
            style::svg_button::text("terminal.svg", size)
                .on_press(Message::HomePaneView(HomePaneViewMessage::Editor(
                    id,
                    EditorViewMessage::ToggleTerminalPanel,
                )))
                .into(),
            style::svg_button::text("search.svg", size)
                .on_press(Message::HomePaneView(HomePaneViewMessage::Editor(
                    id,
                    EditorViewMessage::OpenSearch,
                )))
                .into(),
        ]);

        row(toolbar).spacing(8).into()
    }

    pub fn view<'a>(
        &'a self,
        _app: &'a Application,
        id: u32,
        pane: iced::widget::pane_grid::Pane,
    ) -> Element<'a, Message> {
        let active_tab = self.active_tab();
        let active_terminal = self
            .terminals
            .get(self.active_terminal)
            .or_else(|| self.terminals.first());

        let tabs = scrollable(row(self.tabs.iter().enumerate().map(|(index, tab)| {
            let active = index == self.active_tab;
            let label = if tab.editor.is_modified() {
                format!("{} *", tab.title)
            } else {
                tab.title.clone()
            };
            let close: Element<'_, Message> = if self.tabs.len() > 1 {
                style::svg_button::text("close.svg", BODY_SIZE)
                    .on_press(Message::HomePaneView(HomePaneViewMessage::Editor(
                        id,
                        EditorViewMessage::CloseTab(index),
                    )))
                    .padding(Padding::from([6.0, 4.0]))
                    .into()
            } else {
                space::horizontal().width(1).into()
            };

            container(
                row![
                    button(text(label).size(BODY_SIZE).style(if active {
                        style::text::primary
                    } else {
                        style::text::text
                    }))
                    .style(style::button::transparent_back_white_text)
                    .padding(Padding::from([8.0, 10.0]))
                    .on_press(Message::HomePaneView(
                        HomePaneViewMessage::Editor(id, EditorViewMessage::SelectTab(index))
                    )),
                    close
                ]
                .spacing(6)
                .align_y(Vertical::Center),
            )
            .padding(Padding::from([4.0, 8.0]))
            .style(if active {
                style::container::neutral_back
            } else {
                style::container::chat_back
            })
            .into()
        })))
        .direction(scrollable::Direction::Horizontal(
            scrollable::Scrollbar::default(),
        ))
        .height(Length::Shrink);

        let explorer = container(
            column![
                text(if let Some(path) = &self.folder_path {
                    path.to_string()
                } else {
                    String::from("no working dir")
                })
                .size(BODY_SIZE)
                .style(style::text::translucent::text),
                scrollable(column(
                    self.tree
                        .iter()
                        .map(|node| self.view_tree_node(id, node, 0))
                ))
                .height(Length::Fill),
            ]
            .spacing(10),
        )
        .id(Self::explorer_root_widget_id(id))
        .padding(12)
        .width(self.sidebar_split)
        .style(style::container::side_bar);

        let editor = container(active_tab.editor.view().map(move |msg| {
            Message::HomePaneView(HomePaneViewMessage::Editor(
                id,
                EditorViewMessage::Event(msg),
            ))
        }))
        .padding(6)
        .height(Length::Fill)
        .style(style::container::code_darkened);

        let editor = container(
            column![
                tabs,
                editor,
                container(
                    row![
                        text(
                            active_tab
                                .path
                                .clone()
                                .unwrap_or(String::from("no file open"))
                        )
                        .size(BODY_SIZE)
                        .style(style::text::translucent::text),
                        space::horizontal(),
                        text(format!(
                            "{} lines",
                            active_tab.editor.content().lines().count()
                        ))
                        .size(BODY_SIZE)
                        .style(style::text::translucent::text)
                    ]
                    .spacing(8)
                )
                .padding(2)
            ]
            .spacing(4),
        );

        let terminal_panel: Element<'a, Message> = if self.terminal_panel_open {
            container(
                column![
                    row![
                        scrollable(row(self.terminals.iter().enumerate().map(
                            |(index, term)| {
                                let active = index == self.active_terminal;
                                let close: Element<'_, Message> = if self.terminals.len() > 1 {
                                    style::svg_button::text("close.svg", BODY_SIZE)
                                        .on_press(Message::HomePaneView(
                                            HomePaneViewMessage::Editor(
                                                id,
                                                EditorViewMessage::CloseTerminal(index),
                                            ),
                                        ))
                                        .into()
                                } else {
                                    space::horizontal().width(1).into()
                                };
                                container(
                                    row![
                                        button(text(&term.title).size(BODY_SIZE).style(
                                            if active {
                                                style::text::primary
                                            } else {
                                                style::text::text
                                            }
                                        ))
                                        .style(style::button::transparent_back_white_text)
                                        .padding(Padding::from([6.0, 10.0]))
                                        .on_press(
                                            Message::HomePaneView(HomePaneViewMessage::Editor(
                                                id,
                                                EditorViewMessage::SelectTerminal(index),
                                            ))
                                        ),
                                        close
                                    ]
                                    .spacing(6)
                                    .align_y(Vertical::Center),
                                )
                                .padding(Padding::from(5))
                                .style(if active {
                                    style::container::neutral_back
                                } else {
                                    style::container::chat_back
                                })
                                .into()
                            }
                        )))
                        .direction(scrollable::Direction::Horizontal(
                            scrollable::Scrollbar::default(),
                        ))
                        .spacing(4)
                        .height(Length::Shrink),
                        space::horizontal(),
                        style::svg_button::text("add.svg", BODY_SIZE + 2).on_press(
                            Message::HomePaneView(HomePaneViewMessage::Editor(
                                id,
                                EditorViewMessage::NewTerminal,
                            ))
                        ),
                        style::svg_button::text("close.svg", BODY_SIZE + 2).on_press(
                            Message::HomePaneView(HomePaneViewMessage::Editor(
                                id,
                                EditorViewMessage::ToggleTerminalPanel,
                            ))
                        )
                    ]
                    .align_y(Vertical::Center)
                    .spacing(8),
                    container(
                        active_terminal
                            .map(|term| {
                                TerminalView::show(&term.terminal).map(move |event| {
                                    Message::HomePaneView(HomePaneViewMessage::Editor(
                                        id,
                                        EditorViewMessage::TerminalEvent(event),
                                    ))
                                })
                            })
                            .unwrap_or_else(|| text("No terminal open").into()),
                    )
                    .height(Length::Fill)
                    .padding(6)
                    .style(style::container::code_darkened)
                ]
                .spacing(4),
            )
            .height(Length::Fill)
            .into()
        } else {
            space::vertical().height(1).into()
        };

        let main_area: Element<'a, Message> = if self.terminal_panel_open {
            horizontal_split(
                editor,
                terminal_panel,
                Self::snap_terminal_split(self.terminal_split),
                move |split| {
                    Message::HomePaneView(HomePaneViewMessage::Editor(
                        id,
                        EditorViewMessage::TerminalSplitDragged(split),
                    ))
                },
            )
            .strategy(Strategy::End)
            .into()
        } else {
            editor.into()
        };

        container(
            column![
                container(self.view_toolbar(id, pane))
                    .padding(5)
                    .style(style::container::neutral_back),
                container(
                    vertical_split(
                        explorer,
                        main_area,
                        Self::snap_sidebar_split(self.sidebar_split),
                        move |split| {
                            Message::HomePaneView(HomePaneViewMessage::Editor(
                                id,
                                EditorViewMessage::SidebarSplitDragged(split),
                            ))
                        },
                    )
                    .strategy(Strategy::Start),
                )
                .height(Length::Fill)
            ]
            .spacing(5),
        )
        .height(Length::Fill)
        .into()
    }

    fn tree_node_widget_id(id: u32, path: &str, is_dir: bool) -> WidgetId {
        WidgetId::from(format!(
            "editor:{}:{}:{}",
            id,
            if is_dir { "folder" } else { "file" },
            path
        ))
    }

    fn explorer_root_widget_id(id: u32) -> WidgetId {
        WidgetId::from(format!("editor:{}:explorer-root", id))
    }
}
