use chrono::Utc;

use common::{Comment, Post};
use sqlx::{sqlite::SqliteQueryResult, FromRow, Pool, Row, Sqlite};

/// # Errors
/// See [`sqlx::error::Error`]
pub async fn store_new_post(
    post: &Post,
    post_id: u32,
    db_pool: &Pool<Sqlite>,
) -> std::result::Result<SqliteQueryResult, sqlx::error::Error> {
    sqlx::query("INSERT INTO posts (id, username, content, timestamp) VALUES ($1, $2, $3, $4)")
        .bind(post_id)
        .bind(&post.username)
        .bind(&post.content)
        .bind(post.timestamp)
        .execute(db_pool)
        .await
}

/// # Errors
/// See [`sqlx::error::Error`]
pub async fn store_new_comment(
    comment: &Comment,
    comment_id: u32,
    db_pool: &Pool<Sqlite>,
) -> std::result::Result<SqliteQueryResult, sqlx::error::Error> {
    sqlx::query("INSERT INTO comments (id, post_id, username, content, timestamp) VALUES ($1, $2, $3, $4, $5)")
        .bind(comment_id)
        .bind(comment.post_id)
        .bind(&comment.username)
        .bind(&comment.content)
        .bind(comment.timestamp)
        .execute(db_pool)
        .await
}

/// # Errors
/// See [`sqlx::error::Error`]
pub async fn store_comment(
    comment: &DBComment,
    db_pool: &Pool<Sqlite>,
) -> std::result::Result<SqliteQueryResult, sqlx::error::Error> {
    sqlx::query("INSERT INTO comments (id, post_id, username, content, timestamp) VALUES ($1, $2, $3, $4, $5)")
        .bind(comment.id)
        .bind(comment.post_id)
        .bind(&comment.username)
        .bind(&comment.content)
        .bind(comment.timestamp)
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
    pub timestamp: i64,
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
    pub timestamp: i64,
}

impl DBComment {
    #[must_use]
    pub fn new(id: u32, post_id: u32, username: &str, content: &str) -> Self {
        Self {
            id,
            post_id,
            username: username.to_string(),
            content: content.to_string(),
            timestamp: Utc::now().timestamp(),
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
            timestamp: post.timestamp,

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
            timestamp: comment.timestamp,
        }
    }
}
