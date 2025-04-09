use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FileNode {
    File,
    Directory(HashMap<String, FileNode>),
}

impl FileNode {
    pub fn as_directory_mut(&mut self) -> Option<&mut HashMap<String, FileNode>> {
        if let FileNode::Directory(ref mut dir) = self {
            Some(dir)
        } else {
            None
        }
    }
}
