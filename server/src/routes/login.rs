use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;

use rand::rngs::OsRng;
use sqlx::Pool;
use sqlx::Sqlite;

use common::User;
use server::DBUser;

/// Input: `new_user: Json<User>`
///
/// Output: `(StatusCode, Json<Option<String>>)`
pub async fn route(
    State(db_pool): State<Pool<Sqlite>>,
    Json(input): Json<User>,
) -> (StatusCode, Json<Option<String>>) {
    let fetched_user = sqlx::query_as::<_, DBUser>("SELECT * from users WHERE username = $1")
        .bind(input.username)
        .fetch_one(&db_pool)
        .await;

    let Ok(user) = fetched_user else {
        return (StatusCode::NOT_FOUND, Json(None));
    };

    let hashed: PasswordHash<'_> = PasswordHash::new(&user.hashed_password).unwrap();

    if Argon2::default()
        .verify_password(input.password.as_bytes(), &hashed)
        .is_err()
    {
        return (StatusCode::UNAUTHORIZED, Json(None));
    }

    let new_session_id = SaltString::generate(&mut OsRng).to_string();

    let query = sqlx::query("INSERT OR REPLACE INTO sessions (username, id) VALUES ($1, $2)")
        .bind(user.username)
        .bind(&new_session_id)
        .execute(&db_pool)
        .await;

    match query {
        Ok(_) => (StatusCode::OK, Json(Some(new_session_id))),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(None)),
    }
}
