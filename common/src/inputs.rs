use serde::{Deserialize, Serialize};

/// Used only as an input to an API endpoint
#[derive(Debug, Serialize, Deserialize)]
pub struct InputComment {
    pub post_id: u32,
    pub content: String,
}
