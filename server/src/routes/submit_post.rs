use axum::extract::State;
use axum::headers::{authorization::Bearer, Authorization};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{Json, TypedHeader};

use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};

use sqlx::Pool;
use sqlx::Sqlite;

use common::Post;
use server::{get_last_id, store_comment, store_new_post, DBComment};

pub async fn route(
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
    State(db_pool): State<Pool<Sqlite>>,
    Json(input): Json<Post>,
) -> impl IntoResponse {
    if auth.token() != common::API_KEY {
        return (StatusCode::UNAUTHORIZED, "Wrong bearer".to_string());
    }

    tracing::debug!("recieved {:?}", input);

    let new_post_id: u32 = get_last_id("posts", &db_pool).await + 1;
    let new_comment_id: u32 = get_last_id("comments", &db_pool).await + 1;

    let res = store_new_post(&input, new_post_id, &db_pool).await;

    let loading: DBComment =
        DBComment::new(new_comment_id, new_post_id, "AI", "Loading, please wait!");
    store_comment(&loading, &db_pool).await.unwrap();

    tokio::spawn(async move {
        let response: String = get_advice(&input.content);

        sqlx::query("UPDATE comments SET content = $2 WHERE id = $1")
            .bind(new_comment_id)
            .bind(response)
            .execute(&db_pool)
            .await
            .unwrap();

        tracing::info!("post {}: ai done", new_post_id);
    });

    match res {
        Ok(_) => (StatusCode::ACCEPTED, "OK, reload".to_string()),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, format!("{err}")),
    }
}

fn create_message<'a>(py: Python<'a>, content: &'a str) -> &'a PyDict {
    let message = PyDict::new(py);
    message.set_item("role", "user").unwrap();
    message.set_item("content", content).unwrap();

    message
}

fn get_advice(input: &str) -> String {
    Python::with_gil(|py| {
        let chat: &PyAny = py.import("g4f").unwrap().getattr("ChatCompletion").unwrap();

        let prompt: String = format!(
            r#"Depending on this message "{input}", what advice would you give this person? Keep it concise. Only respond with the advice."#
        );

        let prompt: &PyDict = create_message(py, &prompt);
        let messages: &PyList = PyList::new(py, vec![prompt]);

        let build_args: &PyDict = PyDict::new(py);
        build_args.set_item("messages", messages).unwrap();

        let mut i = 0;
        loop {
            i += 1;
            if i == 5 {
                return "Error".to_string();
            }

            match chat.call_method("create", ("gpt-3.5-turbo",), Some(build_args)) {
                Ok(res) => return res.to_string(),
                Err(err) => {
                    tracing::error!("ai error: {}, retry {i}", err.value(py));
                    continue;
                }
            }
        }
    })
}
