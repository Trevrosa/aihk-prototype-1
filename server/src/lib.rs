use axum::headers::{authorization::Bearer, Authorization};
use chrono::Utc;

use common::{Comment, Post};
use sqlx::{FromRow, Pool, Sqlite};

/// A user sesion
#[derive(Debug, FromRow)]
pub struct DBSession {
    pub username: String,
    pub id: String,
}

/// `User`s are never stored in database. Instead, `DBUser` is used since passwords are hashed before stored.
#[derive(Debug, FromRow)]
pub struct DBUser {
    pub created: i64,

    pub username: String,
    pub hashed_password: String,
}

/// `DBPost`s are individual posts without comments attached to them.
///
/// This is because in the `SQLite` database, posts and comments are stored in different tables; they cannot be stored together.
///
/// `DBPost`s are never sent or recieved since only it is stored in the database.
#[derive(Debug, FromRow)]
pub struct DBPost {
    /// Because an `id` is stored as an `INTEGER PRIMARY KEY`, `id` has to be `u32`
    pub id: u32,
    pub created: i64,

    pub username: String,
    pub content: String,
}

/// `DBComment`s are individual comments with an `id` and `post_id`.
///
/// `DBPost`s are never sent or recieved since only it is stored in the database.
#[derive(Debug, FromRow)]
pub struct DBComment {
    pub id: u32,
    pub post_id: u32,

    pub created: i64,

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
            created: Utc::now().timestamp(),
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
            id: post.id,

            username: post.username,
            content: post.content,
            created: post.created,

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
            id: comment.id,
            post_id: comment.post_id,

            username: comment.username.clone(),
            content: comment.content.clone(),
            created: comment.created,
        }
    }
}

/// Returns [`Ok(DBSession)`] if `session_id` was found in database, otherwise, return [`Err(sqlx::error::Error)`]
///
/// # Errors
///
/// Will always error if header is not found, otherwise, refer to [`sqlx::error::Error`]
pub async fn verify_auth(
    header: &Authorization<Bearer>,
    db_pool: &Pool<Sqlite>,
) -> Result<DBSession, sqlx::error::Error> {
    let res = sqlx::query_as::<_, DBSession>("SELECT * FROM sessions WHERE id = $1")
        .bind(header.token().trim())
        .fetch_one(db_pool)
        .await;

    tracing::debug!("{:#?}", &res);

    res
}
