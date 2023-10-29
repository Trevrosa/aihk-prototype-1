use chrono::Utc;
use serde::{Deserialize, Serialize};

pub mod inputs;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Session {
    pub username: String,
    pub id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    pub created: i64,

    pub username: String,
    pub password: String,
}

impl User {
    #[must_use]
    pub fn new(username: String, password: String) -> Self {
        Self {
            username,
            password,
            created: Utc::now().timestamp(),
        }
    }
}

/// `Post`s are sent and recieved by both `frontend` and `server`.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Post {
    pub id: u32,
    pub created: i64,

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
    pub id: u32,
    pub created: i64,

    /// Comments must have a post_id to be valid.
    pub post_id: u32,

    pub username: String,
    pub content: String,
}
