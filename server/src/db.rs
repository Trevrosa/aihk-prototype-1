use server::{DBComment, DBPost, DBUser};
use sqlx::{sqlite::SqliteQueryResult, Pool, Row, Sqlite};

/// # Errors
/// See [`sqlx::error::Error`]
pub async fn store_new_user(
    user: &DBUser,
    db_pool: &Pool<Sqlite>,
) -> std::result::Result<SqliteQueryResult, sqlx::error::Error> {
    sqlx::query("INSERT INTO users (username, hashed_password, created) VALUES ($1, $2, $3)")
        .bind(&user.username)
        .bind(&user.hashed_password)
        .bind(user.created)
        .execute(db_pool)
        .await
}

/// # Errors
/// See [`sqlx::error::Error`]
pub async fn store_post(
    post: &DBPost,
    db_pool: &Pool<Sqlite>,
) -> std::result::Result<SqliteQueryResult, sqlx::error::Error> {
    sqlx::query("INSERT INTO posts (id, username, content, created) VALUES ($1, $2, $3, $4)")
        .bind(post.id)
        .bind(&post.username)
        .bind(&post.content)
        .bind(post.created)
        .execute(db_pool)
        .await
}

/// # Errors
/// See [`sqlx::error::Error`]
pub async fn store_comment(
    comment: &DBComment,
    db_pool: &Pool<Sqlite>,
) -> std::result::Result<SqliteQueryResult, sqlx::error::Error> {
    sqlx::query("INSERT INTO comments (id, post_id, username, content, created) VALUES ($1, $2, $3, $4, $5)")
        .bind(comment.id)
        .bind(comment.post_id)
        .bind(&comment.username)
        .bind(&comment.content)
        .bind(comment.created)
        .execute(db_pool)
        .await
}

pub async fn get_last_id(table: &str, db_pool: &Pool<Sqlite>) -> u32 {
    sqlx::query(&format!("SELECT id FROM {table} ORDER BY id DESC LIMIT 1"))
        .fetch_one(db_pool)
        .await
        .map_or(0, |row| row.get::<u32, usize>(0))
}
