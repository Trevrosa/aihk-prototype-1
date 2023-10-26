use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

use common::{Comment, Post};
use sqlx::{FromRow, Pool, Sqlite, Row, sqlite::SqliteQueryResult};

pub async fn store_new_post(post: &Post, post_id: u32, db_pool: &Pool<Sqlite>) -> std::result::Result<SqliteQueryResult, sqlx::error::Error> {
    sqlx::query("INSERT INTO posts (id, username, content) VALUES ($1, $2, $3)")
        .bind(post_id)
        .bind(&post.username)
        .bind(&post.content)
        .execute(db_pool)
        .await
}

pub async fn store_new_comment(comment: &Comment, comment_id: u32, db_pool: &Pool<Sqlite>) -> std::result::Result<SqliteQueryResult, sqlx::error::Error> {
    sqlx::query("INSERT INTO comments (id, post_id, username, content) VALUES ($1, $2, $3, $4)")
        .bind(comment_id)
        .bind(comment.post_id)
        .bind(&comment.username)
        .bind(&comment.content)
        .execute(db_pool)
        .await
}

pub async fn store_comment(comment: &DBComment, db_pool: &Pool<Sqlite>) -> std::result::Result<SqliteQueryResult, sqlx::error::Error> {
    sqlx::query("INSERT INTO comments (id, post_id, username, content) VALUES ($1, $2, $3, $4)")
        .bind(comment.id)
        .bind(comment.post_id)
        .bind(&comment.username)
        .bind(&comment.content)
        .execute(db_pool)
        .await
}

pub async fn get_last_id(table: &str, db_pool: &Pool<Sqlite>) -> u32 {
    sqlx::query(&format!("SELECT id FROM {table} ORDER BY id DESC LIMIT 1"))
        .fetch_one(db_pool)
        .await
        .map_or(0, |row| row.get::<u32, usize>(0))
}

/// `DBPost`s are individual posts with an `id`, without comments attached to them.
///
/// This is because in the `SQLite` database, posts and comments are stored in different tables.
///
/// So, they cannot be stored together.
///
/// `DBPost`s are never sent or recieved since only it has access to the database.
#[derive(Debug, FromRow)]
pub struct DBPost {
    /// Because an `id` is stored as an `INTEGER PRIMARY KEY`, `id` has to be `u32`
    pub id: u32,

    pub username: String,
    pub content: String,
}

/// `DBComment`s are individual comments with an `id` and `post_id`.
///
/// `DBPost`s are never sent or recieved, since only it has access to the database.
#[derive(Debug, FromRow)]
pub struct DBComment {
    pub id: u32,
    pub post_id: u32,

    pub username: String,
    pub content: String,
}

impl DBComment {
    #[must_use]
    pub fn new(id: u32, post_id: u32, username: &str, content: &str) -> Self {
        Self {
            id,
            post_id,
            username: username.to_string(),
            content: content.to_string(),
        }
    }
}

/// Convert from owned `DBPost` to `Post` by attaching comments.
pub trait FromDBPost {
    fn from_db(post: DBPost, comments: Option<Vec<Comment>>) -> Self;
}

impl FromDBPost for Post {
    fn from_db(post: DBPost, comments: Option<Vec<Comment>>) -> Self {
        Self {
            id: Some(post.id),

            username: post.username,
            content: post.content,

            comments,
        }
    }
}

/// Convert from `&DBComment` to owned `Comment` by cloning
pub trait FromDBComment {
    fn from_db(comment: &DBComment) -> Self;
}

impl FromDBComment for Comment {
    fn from_db(comment: &DBComment) -> Self {
        Self {
            id: Some(comment.id),
            post_id: comment.post_id,

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
