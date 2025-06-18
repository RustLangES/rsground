use std::collections::HashMap;
use std::sync::atomic::AtomicBool;

use operational_transform::OperationSeq;
use serde::Serialize;
use tokio::sync::{Notify, RwLock, RwLockWriteGuard};

use crate::collab::ot::transform_index;
use crate::utils::{ArcStr, AsyncInto};

use super::UserOperation;

#[derive(Debug, Default)]
pub struct Document {
    /// State modified by critical sections of the code.
    state: RwLock<DocumentState>,
    /// Used to notify clients of new text operations.
    notify: Notify,
    /// Set to true when the document is destroyed.
    killed: AtomicBool,
}

#[derive(Debug, Default)]
pub struct DocumentState {
    operations: Vec<UserOperation>,
    text: String,
    pub cursors: HashMap<ArcStr, Vec<(u32, u32)>>,
}

#[derive(Clone, Debug, Serialize)]
pub struct DocumentInfo {
    pub text: String,
    pub revision: usize,
}

impl AsyncInto<DocumentInfo> for &Document {
    async fn async_into(self) -> DocumentInfo {
        let state = self.state.read().await;
        DocumentInfo {
            text: state.text.clone(),
            revision: state.operations.len(),
        }
    }
}

impl Document {
    pub fn new() -> Self {
        Document::default()
    }

    pub fn new_with(text: String) -> Self {
        Document {
            state: DocumentState {
                text,
                ..Default::default()
            }
            .into(),
            ..Default::default()
        }
    }

    pub async fn fork(&self) -> Self {
        Document {
            state: DocumentState {
                text: self.state.read().await.text.clone(),
                ..Default::default()
            }
            .into(),
            ..Default::default()
        }
    }

    pub async fn text(&self) -> String {
        self.state.read().await.text.clone()
    }

    pub async fn revision(&self) -> usize {
        self.state.read().await.operations.len()
    }

    pub async fn state_mut(&self) -> RwLockWriteGuard<'_, DocumentState> {
        self.state.write().await
    }

    /// Add actions to document history.
    /// - Transform desynchorized actions
    /// - Notify to document listeners
    pub async fn compose(
        &self,
        user_id: ArcStr,
        revision: usize,
        mut operation: OperationSeq,
    ) -> Result<(), String> {
        log::info!(
            "edit: id = {}, revision = {}, base_len = {}, target_len = {}",
            user_id,
            revision,
            operation.base_len(),
            operation.target_len()
        );

        let mut state = self.state.write().await;

        let new_text = {
            let len = state.operations.len();
            if revision > len {
                return Err(format!("got revision {revision}, but current is {len}"));
            }
            for history_op in &state.operations[revision..] {
                operation = operation
                    .transform(&history_op.operation)
                    .map_err(|err| err.to_string())?
                    .0;
            }
            if operation.target_len() > 256 * 1024 {
                return Err(format!(
                    "target length {} is greater than 256 KiB maximum",
                    operation.target_len()
                ));
            }

            operation
                .apply(&state.text)
                .map_err(|err| err.to_string())?
        };

        for (_, data) in state.cursors.iter_mut() {
            for (start, end) in data.iter_mut() {
                *start = transform_index(&operation, *start);
                *end = transform_index(&operation, *end);
            }
        }

        state.operations.push(UserOperation { user_id, operation });
        state.text = new_text;
        self.notify.notify_waiters();

        Ok(())
    }
}
