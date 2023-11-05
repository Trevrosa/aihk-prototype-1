use axum::extract::State;
use axum::headers::{authorization::Bearer, Authorization};

use axum::http::StatusCode;
use axum::Json;
use axum::TypedHeader;

use chrono::Utc;
use common::inputs::InputComment;
use rustrict::{Censor, Type};
use server::DBComment;

use crate::db::{get_last_id, store_comment};
use server::verify_auth;
use sqlx::{Pool, Sqlite};

/// Input: [`InputComment`]
///
/// Output: `(StatusCode, String)`
pub async fn route(
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
    State(db_pool): State<Pool<Sqlite>>,
    Json(input): Json<InputComment>,
) -> (StatusCode, String) {
    let session = verify_auth(&auth, &db_pool).await;
    if session.is_err() {
        return (StatusCode::UNAUTHORIZED, "Wrong bearer".to_string());
    }

    let analysis = Censor::from_str(&input.content).analyze();
    if analysis.is(Type::SEVERE | Type::SEXUAL) {
        return (StatusCode::FORBIDDEN, "Cannot say that".to_string());
    }

    let username = session.unwrap().username;

    tracing::debug!("recieved {:?}", input);

    let next_comment_id: u32 = get_last_id("comments", &db_pool).await + 1;

    let comment = DBComment {
        id: next_comment_id,
        post_id: input.post_id,
        created: Utc::now().timestamp(),
        username: username.clone(),
        content: input.content,
    };

    let res = store_comment(&comment, &db_pool).await;

    match res {
        Ok(_) => (StatusCode::OK, "OK".to_string()),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, format!("{err}")),
    }
}
