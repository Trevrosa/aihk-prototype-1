use axum::{extract::State, http::StatusCode, Json};
use server::{FromDBComment, FromDBPost};
use sqlx::{Pool, Sqlite};

use common::{Comment, Post};
use server::{DBComment, DBPost};

/// Output: `(StatusCode, Json<Option<Vec<Post>>>)`
#[rustfmt::skip]
pub async fn route(
    State(db_pool): State<Pool<Sqlite>>
) -> (StatusCode, Json<Option<Vec<Post>>>) {
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

    if posts.is_empty() {
        (StatusCode::OK, Json(None))
    } else {
        (StatusCode::OK, Json(Some(posts)))
    }
}
