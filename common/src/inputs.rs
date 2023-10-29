use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct InputComment {
    pub post_id: u32,
    pub content: String,
}
