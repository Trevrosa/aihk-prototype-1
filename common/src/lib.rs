use serde::{Deserialize, Serialize};

/// `Post`s are sent and recieved by both `frontend` and `server`.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Post {
    /// Posts can be created from `frontend`,
    /// meaning no `id` is assigned until processed by `server`.
    ///
    /// Therefore, this can be `None`
    pub id: Option<u32>,

    pub username: String,
    pub content: String,

    /// Posts can have no comments.
    pub comments: Option<Vec<Comment>>,
}

/// `Comment`s are sent and recieved by both `frontend` and `server`.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Comment {
    /// Comments can be created from `frontend`,
    /// meaning no `id` is assigned until processed by `server`.
    ///
    /// Therefore, this can be `None`.
    pub id: Option<u32>,

    /// Comments must have a post_id to be valid.
    pub post_id: u32,

    pub username: String,
    pub content: String,
}

impl Post {
    #[must_use]
    pub fn new(username: String, content: String) -> Self {
        Self {
            username,
            content,
            comments: None,
            id: None,
        }
    }
}

impl Default for Post {
    fn default() -> Self {
        Self {
            username: "nobody".to_string(),
            content: "anything".to_string(),
            comments: None,
            id: None,
        }
    }
}
