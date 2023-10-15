use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Post {
    pub username: String,
    pub content: String,
}

impl Post {
    #[must_use]
    pub fn new(username: String, content: String) -> Self {
        Self { username, content }
    }
}

impl std::fmt::Display for Post {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} said: {}", self.username, self.content)
    }
}
