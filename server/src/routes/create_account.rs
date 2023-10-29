use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHash, PasswordVerifier, PasswordHasher};
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;

use chrono::Utc;
use rand::rngs::OsRng;
use sqlx::Pool;
use sqlx::Sqlite;

use common::User;
use server::DBUser;
use sqlx::sqlite::SqliteQueryResult;

use crate::db::store_new_user;

/// Input: [`User`]
///
/// Output: `(StatusCode, Json<Option<String>>)`
pub async fn route(
    State(db_pool): State<Pool<Sqlite>>,
    Json(input): Json<User>,
) -> (StatusCode, Json<Option<String>>) {
    tracing::debug!("recieved {:?}", input);

    let salt: SaltString = SaltString::generate(&mut OsRng);
    let hashed_password: String = Argon2::default()
        .hash_password(&input.password.as_bytes(), &salt)
        .unwrap()
        .to_string();

    let new_user: DBUser = DBUser {
        created: Utc::now().timestamp(),
        username: input.username.clone(),
        hashed_password: hashed_password.clone(),
    };

    let res: Result<SqliteQueryResult, sqlx::Error> = store_new_user(&new_user, &db_pool).await;

    match res {
        Ok(_) => {
            let hashed: PasswordHash<'_> = PasswordHash::new(&hashed_password).unwrap();

            if Argon2::default()
                .verify_password(input.password.as_bytes(), &hashed)
                .is_err()
            {
                return (StatusCode::UNAUTHORIZED, Json(None));
            }
        
            let new_session_id = SaltString::generate(&mut OsRng).to_string();
        
            sqlx::query("INSERT OR REPLACE INTO sessions (username, id) VALUES ($1, $2)")
                .bind(input.username)
                .bind(&new_session_id)
                .execute(&db_pool)
                .await
                .unwrap();
        
            (StatusCode::OK, Json(Some(new_session_id)))
        },
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(None)),
    }
}
