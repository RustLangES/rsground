use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Action {
    Insertion {
        from: usize,
        text: String,
        owner: String,
    },
    Deletion {
        from: usize,
        to: usize,
        owner: String,
    },
}
