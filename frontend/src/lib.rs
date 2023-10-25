use serde::{Deserialize, Serialize};
use wasm_bindgen::JsCast;
use web_sys::{window, Document, HtmlTextAreaElement};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Post {
    pub username: String,
    pub content: String,
    pub comments: Option<Vec<Comment>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Comment {
    pub username: String,
    pub content: String,
}

impl Comment {
    #[must_use]
    pub fn new(username: String, content: String) -> Self {
        Self { username, content }
    }
}

impl Post {
    #[must_use]
    pub fn new(username: String, content: String) -> Self {
        Self {
            username,
            content,
            comments: None,
        }
    }
}

impl std::default::Default for Post {
    fn default() -> Self {
        Post {
            username: "nobody".to_string(),
            content: "anything".to_string(),
            comments: None,
        }
    }
}

impl std::fmt::Display for Post {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} said: {}", self.username, self.content)
    }
}

/// # Panics
/// Panics if no element with id `id` exists
pub fn set_text(id: &str, content: &str) {
    get_document()
        .get_element_by_id(id)
        .unwrap()
        .set_text_content(Some(content));
}

/// # Panics
/// Panics if no element with id `id` exists
#[must_use]
pub fn get_textarea(id: &str) -> String {
    get_document()
        .get_element_by_id(id)
        .unwrap()
        .unchecked_into::<HtmlTextAreaElement>()
        .value()
}

/// # Panics
/// Refer to `web_sys::Window` and `web_sys::Document`
#[must_use]
pub fn get_document() -> Document {
    window().unwrap().document().unwrap()
}
