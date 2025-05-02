use std::collections::HashMap;

use serde::Serialize;
use tokio::sync::Notify;

use super::ot::{apply_actions, transform_actions};
use super::Action;

#[derive(Debug)]
pub struct Document {
    pub buffer: String,
    /// Users can subscribe to document events
    pub notify: Notify,
    pub history: Vec<Action>,
    pub cursors: HashMap<String, Vec<(usize, usize)>>,
}

#[derive(Clone, Debug, Serialize)]
pub struct DocumentInfo {
    pub text: String,
    pub revision: usize,
}

impl From<&Document> for DocumentInfo {
    fn from(value: &Document) -> Self {
        Self {
            text: value.buffer.clone(),
            revision: value.history.len(),
        }
    }
}

impl Document {
    pub fn new() -> Self {
        Document {
            buffer: String::new(),
            notify: Notify::new(),
            history: Vec::new(),
            cursors: HashMap::new(),
        }
    }

    pub fn new_with(buffer: String) -> Self {
        Document {
            buffer,
            notify: Notify::new(),
            history: Vec::new(),
            cursors: HashMap::new(),
        }
    }

    pub fn fork(&self) -> Self {
        Document {
            buffer: self.buffer.clone(),
            history: self.history.clone(),
            notify: Notify::new(),
            cursors: HashMap::new(),
        }
    }

    pub fn revision(&self) -> usize {
        self.history.len()
    }

    /// Add actions to document history.
    /// - Transform desynchorized actions
    /// - Notify to document listeners
    pub fn compose(&mut self, revision: usize, mut actions: Vec<Action>) -> Vec<Action> {
        if revision == self.revision() {
            self.buffer = apply_actions(&self.buffer, &actions);
            self.history.extend(actions.iter().cloned());
            self.notify.notify_waiters();
            return actions;
        } else if revision > self.history.len() {
            log::warn!("Someone comes from the future");
            return Vec::new();
        }

        let desynchronized_history = &self.history[revision..];

        transform_actions(actions.as_mut_slice(), desynchronized_history);

        self.buffer = apply_actions(&self.buffer, &actions);
        self.history.extend_from_slice(&actions);
        self.notify.notify_waiters();

        actions
    }
}
