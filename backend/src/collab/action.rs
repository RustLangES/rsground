use std::sync::Arc;

use operational_transform::OperationSeq;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserOperation {
    pub user_id: Arc<str>,
    pub operation: OperationSeq,
}
