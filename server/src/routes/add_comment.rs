use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;

use common::Comment;

use server::get_last_id;
use server::store_new_comment;
use sqlx::Pool;
use sqlx::Sqlite;

pub async fn route(
    State(db_pool): State<Pool<Sqlite>>,
    Json(input): Json<Comment>,
) -> impl IntoResponse {
    tracing::debug!("recieved {:?}", input);

    let next_comment_id: u32 = get_last_id("comments", &db_pool).await + 1;
    let res = store_new_comment(&input, next_comment_id, &db_pool).await;

    match res {
        Ok(_) => (StatusCode::ACCEPTED, "OK".to_string()),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, format!("{err}")),
    }
}