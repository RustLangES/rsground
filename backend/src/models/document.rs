use cola::{Deletion, Replica, ReplicaId};
use log::info;
use serde::{Deserialize, Serialize};
use std::ops::Range;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Action {
    Insertion {
        pos: usize,
        text: String,
        timestamp: u64,
    },
    Deletion {
        range_start: usize,
        range_end: usize,
        timestamp: u64,
    },
}

pub struct Insertion {
    pub text: String,
    pub crdt: cola::Insertion,
}

use std::sync::atomic::{AtomicU64, Ordering};

// Define un contador global para ReplicaId.
static REPLICA_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

pub fn generate_unique_replica_id() -> u64 {
    REPLICA_ID_COUNTER.fetch_add(1, Ordering::SeqCst)
}

#[derive(Clone, Debug)]
pub struct Document {
    pub(crate) buffer: String,
    pub(crate) crdt: Replica,
    pub(crate) replica_id: ReplicaId,
    pub(crate) history: Vec<Action>,
    pub(crate) current_timestamp: u64,
}

impl Document {
    pub fn new<S: Into<String>>(text: S, replica_id: ReplicaId) -> Self {
        let buffer = text.into();
        let crdt = Replica::new(replica_id, buffer.len());
        info!("Creando nuevo documento con buffer: '{}'", buffer);
        Document {
            buffer,
            crdt,
            replica_id,
            history: Vec::new(),
            current_timestamp: 0,
        }
    }

    pub fn fork(&self, new_replica_id: ReplicaId) -> Self {
        let new_replica_id = if new_replica_id == self.replica_id {
            generate_unique_replica_id()
        } else {
            new_replica_id
        };
        let crdt = self.crdt.fork(new_replica_id);
        Document {
            buffer: self.buffer.clone(),
            crdt,
            replica_id: new_replica_id,
            history: self.history.clone(),
            current_timestamp: self.current_timestamp,
        }
    }

    pub fn insert<S: Into<String>>(&mut self, insert_at: usize, text: S) -> Insertion {
        let text = text.into();
        info!(
            "Insertando texto '{}' en la posición {} del documento",
            text, insert_at
        );
        self.buffer.insert_str(insert_at, &text);
        let insertion = self.crdt.inserted(insert_at, text.len());
        self.current_timestamp += 1;
        let action = Action::Insertion {
            pos: insert_at,
            text: text.clone(),
            timestamp: self.current_timestamp,
        };
        self.history.push(action);
        info!("Buffer actualizado: '{}'", self.buffer);
        Insertion {
            text,
            crdt: insertion,
        }
    }

    pub fn delete(&mut self, range: Range<usize>) -> Deletion {
        info!(
            "Eliminando rango {}..{} en el documento",
            range.start, range.end
        );
        self.buffer.replace_range(range.clone(), "");
        let deletion = self.crdt.deleted(range.clone());
        self.current_timestamp += 1;
        let action = Action::Deletion {
            range_start: range.start,
            range_end: range.end,
            timestamp: self.current_timestamp,
        };
        self.history.push(action);
        info!("Buffer después de eliminación: '{}'", self.buffer);
        deletion
    }

    pub fn get_operations_since(&self, last_timestamp: u64) -> Vec<Action> {
        info!("Obteniendo operaciones con timestamp > {}", last_timestamp);
        self.history
            .iter()
            .filter(|action| match action {
                Action::Insertion { timestamp, .. } => *timestamp > last_timestamp,
                Action::Deletion { timestamp, .. } => *timestamp > last_timestamp,
            })
            .cloned()
            .collect()
    }
}
