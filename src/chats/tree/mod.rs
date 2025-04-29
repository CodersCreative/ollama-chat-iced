pub mod view;

use super::chat::Chat;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChatNode {
    pub chat: Chat,
    pub reason: Option<Reason>,
    pub children: Vec<ChatNode>,
    pub selected_child_index: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Reason {
    Model(String),
    Regeneration,
    Sibling,
}

impl ChatNode {
    pub fn add_chat(&mut self, chat: Chat, reason: Option<Reason>) {
        if let None = self.selected_child_index {
            self.selected_child_index = Some(0);
        }

        let new_node = ChatNode {
            chat,
            reason,
            children: Vec::new(),
            selected_child_index: None,
        };

        self.children.push(new_node);
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChatTree {
    pub root: ChatNode,
}

impl ChatTree {
    pub fn new(root_chat: Chat) -> Self {
        ChatTree {
            root: ChatNode {
                chat: root_chat,
                reason: None,
                children: Vec::new(),
                selected_child_index: None,
            },
        }
    }
    pub fn add_chat(&mut self, chat: Chat, reason: Option<Reason>) {
        if let Some(last) = self.get_last_mut() {
            last.add_chat(chat, reason);
        }
    }

    pub fn get_last_mut(&mut self) -> Option<&mut ChatNode> {
        self.get_node_mut_with_path(&self.get_full_path())
    }

    pub fn get_last(&self) -> Option<&ChatNode> {
        self.get_node_with_path(&self.get_full_path())
    }

    pub fn get_last_parent_mut(&mut self) -> Option<&mut ChatNode> {
        let mut path = self.get_full_path();
        path.pop();
        self.get_node_mut_with_path(&path)
    }

    pub fn get_last_parent(&self) -> Option<&ChatNode> {
        let mut path = self.get_full_path();
        path.pop();
        self.get_node_with_path(&path)
    }

    pub fn get_node_mut_with_path(&mut self, path: &Vec<usize>) -> Option<&mut ChatNode> {
        let mut current_node = &mut self.root;
        for &index in path {
            if index < current_node.children.len() {
                current_node = &mut current_node.children[index];
            } else {
                return None;
            }
        }
        Some(current_node)
    }

    pub fn get_node_with_path(&self, path: &Vec<usize>) -> Option<&ChatNode> {
        let mut current_node = &self.root;
        for &index in path {
            if index < current_node.children.len() {
                current_node = &current_node.children[index];
            } else {
                return None;
            }
        }
        Some(current_node)
    }

    pub fn get_node_mut_from_index(&mut self, index: usize) -> Option<&mut ChatNode> {
        self.get_node_mut_with_path(&self.get_path_to_index(index))
    }

    pub fn get_node_from_index(&self, index: usize) -> Option<&ChatNode> {
        self.get_node_with_path(&self.get_path_to_index(index))
    }

    pub fn select_child_with_path(
        &mut self,
        path: &Vec<usize>,
        child_index: usize,
    ) -> Result<(), String> {
        let parent_node = self.get_node_mut_with_path(path).ok_or("Invalid path")?;
        if child_index < parent_node.children.len() {
            parent_node.selected_child_index = Some(child_index);
            Ok(())
        } else {
            Err("Child index out of bounds".to_string())
        }
    }

    pub fn select_child_from_index(
        &mut self,
        index: usize,
        child_index: usize,
    ) -> Result<(), String> {
        self.select_child_with_path(&self.get_path_to_index(index), child_index)
    }

    pub fn get_selected_child_with_path(&self, path: &Vec<usize>) -> Option<&ChatNode> {
        let parent_node = self.get_node_with_path(path)?;
        match parent_node.selected_child_index {
            Some(index) => parent_node.children.get(index),
            None => None,
        }
    }

    pub fn get_selected_child_mut_with_path(&mut self, path: &Vec<usize>) -> Option<&mut ChatNode> {
        let parent_node = self.get_node_mut_with_path(path)?;
        match parent_node.selected_child_index {
            Some(index) => parent_node.children.get_mut(index),
            None => None,
        }
    }

    pub fn get_selected_child_from_index(&self, index: usize) -> Option<&ChatNode> {
        self.get_selected_child_with_path(&self.get_path_to_index(index))
    }

    pub fn get_selected_child_mut_from_index(&mut self, index: usize) -> Option<&mut ChatNode> {
        self.get_selected_child_mut_with_path(&self.get_path_to_index(index))
    }

    pub fn get_path_to_index(&self, index: usize) -> Vec<usize> {
        let mut current_node = &self.root;
        let mut path = Vec::new();

        while let Some(child_index) = current_node.selected_child_index {
            if !path.is_empty() {
                if (path.len() - 1) >= index {
                    break;
                }
            }

            if let Some(next_node) = current_node.children.get(child_index) {
                current_node = next_node;
                path.push(child_index);
            } else {
                break;
            }
        }
        path
    }

    pub fn get_full_path(&self) -> Vec<usize> {
        let mut current_node = &self.root;
        let mut path = Vec::new();

        while let Some(child_index) = current_node.selected_child_index {
            if let Some(next_node) = current_node.children.get(child_index) {
                current_node = next_node;
                path.push(child_index);
            } else {
                break;
            }
        }
        path
    }

    pub fn get_full_history(&self) -> Vec<&Chat> {
        let mut history = Vec::new();
        let mut current_node = &self.root;
        // history.push(&current_node.chat);

        while let Some(child_index) = current_node.selected_child_index {
            if let Some(next_node) = current_node.children.get(child_index) {
                history.push(&next_node.chat);
                current_node = next_node;
            } else {
                break;
            }
        }
        history
    }
}
pub struct ChatTreeIterator<'a> {
    current_node: &'a ChatNode,
}

impl<'a> Iterator for ChatTreeIterator<'a> {
    type Item = &'a ChatNode;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(selected_index) = self.current_node.selected_child_index {
            if selected_index < self.current_node.children.len() {
                self.current_node = &self.current_node.children[selected_index];
                return Some(self.current_node);
            }
        }
        None
    }
}

impl<'a> IntoIterator for &'a ChatTree {
    type Item = &'a ChatNode;
    type IntoIter = ChatTreeIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        ChatTreeIterator {
            current_node: &self.root,
        }
    }
}
