use axum::{
    extract::State,
    headers::{authorization::Bearer, Authorization},
    http::StatusCode,
    Json, TypedHeader,
};
use server::verify_auth;
use sqlx::{Pool, Sqlite};

pub async fn route(
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
    State(db_pool): State<Pool<Sqlite>>,
) -> (StatusCode, Json<Option<String>>) {
    let session = verify_auth(&auth, &db_pool).await;

    if let Ok(session) = session {
        (StatusCode::OK, Json(Some(session.username)))
    } else {
        (StatusCode::UNAUTHORIZED, Json(None))
    }
}
