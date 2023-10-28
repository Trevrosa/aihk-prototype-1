use axum::{
    extract::State,
    headers::{authorization::Bearer, Authorization},
    http::StatusCode,
    response::IntoResponse,
    Json, TypedHeader,
};
use server::{verify_auth, FromDBComment, FromDBPost};
use sqlx::{Pool, Sqlite};

use common::{Comment, Post};
use server::{DBComment, DBPost};

pub async fn route(
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
    State(db_pool): State<Pool<Sqlite>>,
) -> impl IntoResponse {
    if !verify_auth(&auth) {
        return (StatusCode::UNAUTHORIZED, Json(vec![Post::default()]));
    }

    let db_posts: Vec<DBPost> = sqlx::query_as::<_, DBPost>("SELECT * from posts")
        .fetch_all(&db_pool)
        .await
        .unwrap();
    let db_comments: Vec<DBComment> = sqlx::query_as::<_, DBComment>("SELECT * from comments")
        .fetch_all(&db_pool)
        .await
        .unwrap();

    let mut posts: Vec<Post> = Vec::with_capacity(db_posts.len());

    for db_post in db_posts {
        let comments: Vec<Comment> = db_comments
            .iter()
            .filter(|c| c.post_id == db_post.id)
            .map(Comment::from_db)
            .collect::<Vec<Comment>>();

        let comments: Option<Vec<Comment>> = if comments.is_empty() {
            None
        } else {
            Some(comments)
        };

        posts.push(Post::from_db(db_post, comments));
    }

    tracing::debug!("got: {:#?}", posts);

    (StatusCode::OK, Json(posts))
}
