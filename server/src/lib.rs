use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, FromRow)]
pub struct DBPost {
    pub username: String,
    pub content: String,
    pub id: u32,
}

#[derive(Debug, FromRow)]
pub struct DBComment {
    pub username: String,
    pub content: String,
    pub post_id: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Post {
    pub username: String,
    pub content: String,
    pub comments: Option<Vec<Comment>>,
}

impl Post {
    #[must_use]
    pub fn from_db(post: DBPost, comments: Option<Vec<Comment>>) -> Self {
        Self {
            username: post.username,
            content: post.content,
            comments: comments,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Comment {
    pub username: String,
    pub content: String,
}

impl Comment {
    #[must_use]
    pub fn from_db(comment: &DBComment) -> Self {
        Self {
            username: comment.username.clone(),
            content: comment.content.clone(),
        }
    }
}

// Make our own error that wraps `anyhow::Error`.
pub struct AppError(anyhow::Error);

// Tell axum how to convert `AppError` into a response.
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", self.0),
        )
            .into_response()
    }
}

// This enables using `?` on functions that return `Result<_, anyhow::Error>` to turn them into `Result<_, AppError>`
impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

// Allow use like `anyhow::Result<T>`
pub type Result<T> = core::result::Result<T, AppError>;
