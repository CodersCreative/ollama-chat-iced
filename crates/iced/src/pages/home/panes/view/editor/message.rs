use crate::{Application, InputMessage, Message, pages::home::panes::view::HomePaneViewMessage};
use iced::Task;
use iced::widget::Id as WidgetId;
use iced_code_editor::Message as CodeEditorMessage;
use iced_drop::zones_on_point;
use iced_term::{
    Command as TerminalCommand, Event as TerminalEvent, actions::Action as TerminalAction,
};
use std::{collections::HashMap, path::Path};

use super::*;

#[derive(Debug, Clone)]
pub enum EditorViewMessage {
    Event(CodeEditorMessage),
    OpenFile,
    OpenFolder,
    CreateFileRoot,
    CreateFolderRoot,
    CreateFileIn(String),
    CreateFolderIn(String),
    Save,
    SaveAs,
    OpenSearch,
    ToggleFolder(String),
    RenameStart(String),
    RenameInput(String, InputMessage),
    RenameSubmit(String),
    DeletePath(String),
    RefreshTree,
    SidebarSplitDragged(f32),
    TerminalSplitDragged(f32),
    DragMove(String, iced::Point),
    Drop(String, iced::Point),
    DropZones(String, Vec<(WidgetId, iced::Rectangle)>),
    CancelDrag,
    PathRenamed(String, String),
    PathDeleted(String),
    PathCreated(String),
    ToggleTerminalPanel,
    NewTerminal,
    SelectTerminal(usize),
    CloseTerminal(usize),
    TerminalEvent(TerminalEvent),
    SelectTab(usize),
    CloseTab(usize),
    OpenEntry(String),
    FileLoaded(String, String),
    FolderLoaded(String, Vec<FileTreeNode>),
    FileSaved(String),
}

impl EditorViewMessage {
    pub fn handle(self, app: &mut Application, id: u32) -> Task<Message> {
        match self {
            Self::Event(message) => {
                let task = app
                    .view_data
                    .home
                    .editors
                    .get_mut(&id)
                    .unwrap()
                    .active_tab_mut()
                    .editor
                    .update(&message);

                task.map(move |msg| {
                    Message::HomePaneView(HomePaneViewMessage::Editor(
                        id,
                        EditorViewMessage::Event(msg),
                    ))
                })
            }
            Self::OpenFile => Task::future(async move {
                match EditorView::pick_file().await {
                    Ok((path, content)) => Message::HomePaneView(HomePaneViewMessage::Editor(
                        id,
                        EditorViewMessage::FileLoaded(path, content),
                    )),
                    Err(e) => Message::Err(e),
                }
            }),
            Self::OpenFolder => Task::future(async move {
                match EditorView::pick_folder().await {
                    Ok((path, files)) => Message::HomePaneView(HomePaneViewMessage::Editor(
                        id,
                        EditorViewMessage::FolderLoaded(path, files),
                    )),
                    Err(e) => Message::Err(e),
                }
            }),
            Self::OpenEntry(path) => Task::future(async move {
                match tokio::fs::read_to_string(&path).await {
                    Ok(content) => Message::HomePaneView(HomePaneViewMessage::Editor(
                        id,
                        EditorViewMessage::FileLoaded(path, content),
                    )),
                    Err(e) => Message::Err(e.to_string()),
                }
            }),
            Self::CreateFileRoot => {
                let Some(root) = app
                    .view_data
                    .home
                    .editors
                    .get(&id)
                    .unwrap()
                    .folder_path
                    .clone()
                else {
                    return Task::none();
                };

                Task::future(async move {
                    match EditorView::create_path(root, false).await {
                        Ok(path) => Message::HomePaneView(HomePaneViewMessage::Editor(
                            id,
                            EditorViewMessage::PathCreated(path),
                        )),
                        Err(e) => Message::Err(e),
                    }
                })
            }
            Self::CreateFolderRoot => {
                let Some(root) = app
                    .view_data
                    .home
                    .editors
                    .get(&id)
                    .unwrap()
                    .folder_path
                    .clone()
                else {
                    return Task::none();
                };

                Task::future(async move {
                    match EditorView::create_path(root, true).await {
                        Ok(path) => Message::HomePaneView(HomePaneViewMessage::Editor(
                            id,
                            EditorViewMessage::PathCreated(path),
                        )),
                        Err(e) => Message::Err(e),
                    }
                })
            }
            Self::CreateFileIn(parent) => Task::future(async move {
                match EditorView::create_path(parent, false).await {
                    Ok(path) => Message::HomePaneView(HomePaneViewMessage::Editor(
                        id,
                        EditorViewMessage::PathCreated(path),
                    )),
                    Err(e) => Message::Err(e),
                }
            }),
            Self::CreateFolderIn(parent) => Task::future(async move {
                match EditorView::create_path(parent, true).await {
                    Ok(path) => Message::HomePaneView(HomePaneViewMessage::Editor(
                        id,
                        EditorViewMessage::PathCreated(path),
                    )),
                    Err(e) => Message::Err(e),
                }
            }),
            Self::FolderLoaded(path, files) => {
                let view = app.view_data.home.editors.get_mut(&id).unwrap();
                view.folder_path = Some(path);
                view.tree = files;
                view.collapsed.clear();
                view.editing.clear();
                Task::none()
            }
            Self::ToggleFolder(path) => {
                let view = app.view_data.home.editors.get_mut(&id).unwrap();
                if view.collapsed.contains(&path) {
                    view.collapsed.retain(|x| x != &path);
                } else {
                    view.collapsed.push(path);
                }
                Task::none()
            }
            Self::RenameStart(path) => {
                let view = app.view_data.home.editors.get_mut(&id).unwrap();
                view.editing
                    .insert(path.clone(), EditorView::entry_name(&path));
                Task::none()
            }
            Self::RenameInput(path, InputMessage::Update(value)) => {
                let view = app.view_data.home.editors.get_mut(&id).unwrap();
                view.editing.insert(path, value);
                Task::none()
            }
            Self::RenameInput(_, _) => Task::none(),
            Self::RenameSubmit(path) => {
                let new_name = app
                    .view_data
                    .home
                    .editors
                    .get_mut(&id)
                    .unwrap()
                    .editing
                    .remove(&path)
                    .unwrap_or_else(|| EditorView::entry_name(&path));

                Task::future(async move {
                    let from = Path::new(&path);
                    let parent = from.parent().ok_or_else(|| "No parent folder".to_string());
                    match parent {
                        Ok(parent) => {
                            let to = parent.join(new_name);
                            match tokio::fs::rename(&path, &to).await {
                                Ok(_) => Message::HomePaneView(HomePaneViewMessage::Editor(
                                    id,
                                    EditorViewMessage::PathRenamed(
                                        path.clone(),
                                        to.to_string_lossy().to_string(),
                                    ),
                                )),
                                Err(e) => Message::Err(e.to_string()),
                            }
                        }
                        Err(e) => Message::Err(e),
                    }
                })
            }
            Self::DeletePath(path) => Task::future(async move {
                let result = if Path::new(&path).is_dir() {
                    tokio::fs::remove_dir_all(&path)
                        .await
                        .map_err(|e| e.to_string())
                } else {
                    tokio::fs::remove_file(&path)
                        .await
                        .map_err(|e| e.to_string())
                };

                match result {
                    Ok(_) => Message::HomePaneView(HomePaneViewMessage::Editor(
                        id,
                        EditorViewMessage::PathDeleted(path),
                    )),
                    Err(e) => Message::Err(e),
                }
            }),
            Self::RefreshTree => {
                let view = app.view_data.home.editors.get_mut(&id).unwrap();
                view.dragging = None;
                if let Some(root) = view.folder_path.clone() {
                    let _ = view.refresh_tree();
                    if !Path::new(&root).exists() {
                        view.folder_path = None;
                        view.tree.clear();
                    }
                }
                Task::none()
            }
            Self::SidebarSplitDragged(split) => {
                let view = app.view_data.home.editors.get_mut(&id).unwrap();
                view.sidebar_split = EditorView::snap_sidebar_split(split);
                Task::none()
            }
            Self::TerminalSplitDragged(split) => {
                let view = app.view_data.home.editors.get_mut(&id).unwrap();
                view.terminal_split = EditorView::snap_terminal_split(split);
                Task::none()
            }
            Self::DragMove(path, _) => {
                app.view_data.home.editors.get_mut(&id).unwrap().dragging = Some(path.clone());
                Task::none()
            }
            Self::Drop(path, point) => zones_on_point(
                move |zones| {
                    Message::HomePaneView(HomePaneViewMessage::Editor(
                        id,
                        EditorViewMessage::DropZones(path.clone(), zones),
                    ))
                },
                point,
                None,
                None,
            ),
            Self::DropZones(from, zones) => {
                let view = app.view_data.home.editors.get_mut(&id).unwrap();
                view.dragging = None;

                let mut zone_map = HashMap::new();
                if let Some(root) = view.folder_path.clone() {
                    zone_map.insert(EditorView::explorer_root_widget_id(id), root);
                }
                for node in &view.tree {
                    collect_drop_zones(id, node, &mut zone_map);
                }

                let target = zones
                    .into_iter()
                    .find_map(|(zone_id, _)| zone_map.get(&zone_id).cloned());

                let Some(target) = target else {
                    return Task::none();
                };

                if from == target
                    || target.starts_with(&(from.clone() + std::path::MAIN_SEPARATOR_STR))
                {
                    return Task::none();
                }

                let to_dir = if Path::new(&target).is_dir() {
                    target.clone()
                } else if let Some(parent) = Path::new(&target).parent() {
                    parent.to_string_lossy().to_string()
                } else {
                    return Task::none();
                };

                let file_name = Path::new(&from)
                    .file_name()
                    .and_then(|x| x.to_str())
                    .unwrap_or_default()
                    .to_string();
                let destination = Path::new(&to_dir).join(file_name);

                if destination == Path::new(&from) {
                    return Task::none();
                }

                return Task::future(async move {
                    match tokio::fs::rename(&from, &destination).await {
                        Ok(_) => Message::HomePaneView(HomePaneViewMessage::Editor(
                            id,
                            EditorViewMessage::PathRenamed(
                                from,
                                destination.to_string_lossy().to_string(),
                            ),
                        )),
                        Err(e) => Message::Err(e.to_string()),
                    }
                });
            }
            Self::PathRenamed(from, to) => {
                let view = app.view_data.home.editors.get_mut(&id).unwrap();
                view.dragging = None;
                view.editing.remove(&from);
                view.rename_tab_paths(&from, &to);
                let _ = view.refresh_tree();
                Task::none()
            }
            Self::PathCreated(path) => {
                let view = app.view_data.home.editors.get_mut(&id).unwrap();
                if let Some(parent) = Path::new(&path).parent().and_then(|p| p.to_str()) {
                    view.collapsed.retain(|x| x != parent);
                }
                view.editing
                    .insert(path.clone(), EditorView::entry_name(&path));
                let _ = view.refresh_tree();
                Task::none()
            }
            Self::ToggleTerminalPanel => {
                let view = app.view_data.home.editors.get_mut(&id).unwrap();
                view.terminal_panel_open = !view.terminal_panel_open;
                Task::none()
            }
            Self::NewTerminal => {
                let cwd = app
                    .view_data
                    .home
                    .editors
                    .get(&id)
                    .unwrap()
                    .folder_path
                    .clone();
                match EditorView::new_terminal(cwd) {
                    Ok(term) => {
                        let view = app.view_data.home.editors.get_mut(&id).unwrap();
                        view.terminals.push(term);
                        view.active_terminal = view.terminals.len() - 1;
                        view.terminal_panel_open = true;
                    }
                    Err(e) => return Task::done(Message::Err(e)),
                }
                Task::none()
            }
            Self::SelectTerminal(index) => {
                let view = app.view_data.home.editors.get_mut(&id).unwrap();
                if index < view.terminals.len() {
                    view.active_terminal = index;
                }
                Task::none()
            }
            Self::CloseTerminal(index) => {
                let view = app.view_data.home.editors.get_mut(&id).unwrap();
                if view.terminals.len() > 1 && index < view.terminals.len() {
                    view.terminals.remove(index);
                    view.active_terminal = view
                        .active_terminal
                        .min(view.terminals.len().saturating_sub(1));
                }
                Task::none()
            }
            Self::TerminalEvent(TerminalEvent::BackendCall(term_id, cmd)) => {
                let view = app.view_data.home.editors.get_mut(&id).unwrap();
                let Some(term) = view.terminals.iter_mut().find(|term| term.id == term_id) else {
                    return Task::none();
                };

                match term.terminal.handle(TerminalCommand::ProxyToBackend(cmd)) {
                    TerminalAction::Shutdown => {
                        if let Some(index) =
                            view.terminals.iter().position(|term| term.id == term_id)
                            && view.terminals.len() > 1
                        {
                            view.terminals.remove(index);
                            view.active_terminal = view
                                .active_terminal
                                .min(view.terminals.len().saturating_sub(1));
                        }
                    }
                    _ => {}
                }

                Task::none()
            }
            Self::PathDeleted(path) => {
                let view = app.view_data.home.editors.get_mut(&id).unwrap();
                view.dragging = None;
                view.editing.remove(&path);
                view.close_tabs_for_path(&path);
                let _ = view.refresh_tree();
                Task::none()
            }
            Self::CancelDrag => {
                let view = app.view_data.home.editors.get_mut(&id).unwrap();
                view.dragging = None;
                if let Some(root) = view.folder_path.clone()
                    && let Ok(tree) = EditorView::read_folder_entries(&root)
                {
                    view.tree = tree;
                }
                Task::none()
            }
            Self::FileLoaded(path, content) => {
                let theme = app.theme();
                let view = app.view_data.home.editors.get_mut(&id).unwrap();

                if let Some(index) = view
                    .tabs
                    .iter()
                    .position(|tab| tab.path.as_deref() == Some(path.as_str()))
                {
                    view.active_tab = index;
                    let folder_path = view.folder_path.clone();
                    let tab = &mut view.tabs[index];
                    tab.title = EditorView::title_from_path(&path);
                    tab.path = Some(path.clone());
                    let _ = EditorView::sync_tab_lsp_for_folder(folder_path.as_deref(), tab);
                    return tab.editor.reset(&content).map(move |msg| {
                        Message::HomePaneView(HomePaneViewMessage::Editor(
                            id,
                            EditorViewMessage::Event(msg),
                        ))
                    });
                }

                let mut tab = EditorTab {
                    title: EditorView::title_from_path(&path),
                    path: Some(path.clone()),
                    editor: EditorView::build_editor(
                        &content,
                        EditorView::syntax_from_path(&path),
                        &theme,
                    ),
                };
                let _ = EditorView::sync_tab_lsp_for_folder(view.folder_path.as_deref(), &mut tab);
                view.tabs.push(tab);
                view.active_tab = view.tabs.len() - 1;
                Task::none()
            }
            Self::Save => {
                let view = app.view_data.home.editors.get(&id).unwrap();
                let tab = view.active_tab();
                let content = tab.editor.content();

                if let Some(path) = tab.path.clone() {
                    Task::future(async move {
                        match EditorView::save_file(path, content).await {
                            Ok(path) => Message::HomePaneView(HomePaneViewMessage::Editor(
                                id,
                                EditorViewMessage::FileSaved(path),
                            )),
                            Err(e) => Message::Err(e),
                        }
                    })
                } else {
                    Task::done(Message::HomePaneView(HomePaneViewMessage::Editor(
                        id,
                        EditorViewMessage::SaveAs,
                    )))
                }
            }
            Self::SaveAs => {
                let view = app.view_data.home.editors.get(&id).unwrap();
                let tab = view.active_tab();
                let content = tab.editor.content();
                let filename = tab.title.clone();

                Task::future(async move {
                    match EditorView::pick_save_path(content, filename).await {
                        Ok(path) => Message::HomePaneView(HomePaneViewMessage::Editor(
                            id,
                            EditorViewMessage::FileSaved(path),
                        )),
                        Err(e) => Message::Err(e),
                    }
                })
            }
            Self::FileSaved(path) => {
                let view = app.view_data.home.editors.get_mut(&id).unwrap();
                let title = EditorView::title_from_path(&path);
                let folder_path = view.folder_path.clone();
                let tab = view.active_tab_mut();
                tab.path = Some(path.clone());
                tab.title = title;
                let _ = EditorView::sync_tab_lsp_for_folder(folder_path.as_deref(), tab);
                tab.editor.mark_saved();
                view.notify_active_tab_saved();
                let _ = view.refresh_tree();
                Task::none()
            }
            Self::OpenSearch => app
                .view_data
                .home
                .editors
                .get_mut(&id)
                .unwrap()
                .active_tab_mut()
                .editor
                .open_search_dialog()
                .map(move |msg| {
                    Message::HomePaneView(HomePaneViewMessage::Editor(
                        id,
                        EditorViewMessage::Event(msg),
                    ))
                }),
            Self::SelectTab(index) => {
                let view = app.view_data.home.editors.get_mut(&id).unwrap();
                if index < view.tabs.len() {
                    view.active_tab = index;
                    let active = view.active_tab;
                    let folder_path = view.folder_path.clone();
                    let mut tabs = std::mem::take(&mut view.tabs);
                    if let Some(tab) = tabs.get_mut(active) {
                        let _ = EditorView::sync_tab_lsp_for_folder(folder_path.as_deref(), tab);
                    }
                    view.tabs = tabs;
                }
                Task::none()
            }
            Self::CloseTab(index) => {
                let view = app.view_data.home.editors.get_mut(&id).unwrap();
                if view.tabs.len() > 1 && index < view.tabs.len() {
                    view.tabs.remove(index);
                    view.active_tab = view.active_tab.min(view.tabs.len() - 1);
                }
                Task::none()
            }
        }
    }
}

fn collect_drop_zones(id: u32, node: &FileTreeNode, zone_map: &mut HashMap<WidgetId, String>) {
    if node.is_dir {
        zone_map.insert(
            EditorView::tree_node_widget_id(id, &node.path, true),
            node.path.clone(),
        );
    }

    for child in &node.children {
        collect_drop_zones(id, child, zone_map);
    }
}
