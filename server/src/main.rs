use axum::body::Body;
use axum::extract::State;
use axum::http::{Request, StatusCode};
use axum::response::Html;
use axum::Json;
use axum::{
    response::IntoResponse,
    routing::{get, post},
    Router,
};

use clap::Parser;
use pyo3::prelude::*;

use pyo3::types::{PyDict, PyList};
use server::{Comment, DBComment, DBPost, Post, Result};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::{ConnectOptions, Pool, Row, Sqlite};

use std::env;
use std::net::{IpAddr, Ipv6Addr, SocketAddr};
use std::path::PathBuf;
use std::str::FromStr;

use tokio::fs;
use tower::{ServiceBuilder, ServiceExt};
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;

#[allow(clippy::unused_async)]
#[derive(Parser, Debug)]
#[clap(name = "server", about = "Set server opts")]
struct Opt {
    /// set the log level
    #[clap(short = 'l', long = "log", default_value = "debug")]
    log_level: String,

    /// set the listen port
    #[clap(short = 'p', long = "port", default_value = "8080")]
    port: u16,

    /// set the directory where static files are to be found
    #[clap(long = "static-dir", default_value = "../dist")]
    static_dir: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let opt = Opt::parse();

    // Setup logging & RUST_LOG from args
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", format!("{},hyper=info,mio=info", opt.log_level));
    }

    // enable console logging
    tracing_subscriber::fmt::init();

    let db_path = if std::env::current_dir()?.ends_with("server") {
        "sqlite://all.db"
    } else {
        "sqlite://server/all.db"
    };

    {
        let mut sqlite_connection = SqliteConnectOptions::new()
            .filename(db_path.split("//").nth(1).unwrap())
            .create_if_missing(true)
            .connect()
            .await?;

        sqlx::query("CREATE TABLE IF NOT EXISTS posts (id INTEGER PRIMARY KEY, username TEXT NOT NULL, content TEXT NOT NULL)")
            .execute(&mut sqlite_connection)
            .await?;

        sqlx::query("CREATE TABLE IF NOT EXISTS comments (id INTEGER PRIMARY KEY, post_id INTEGER NOT NULL, username TEXT NOT NULL, content TEXT NOT NULL)")
            .execute(&mut sqlite_connection)
            .await?;

        tracing::debug!("db,table exists");
    }

    let db_pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(db_path)
        .await?;

    tracing::debug!("db pool ready");

    let app = Router::new()
        .route("/api/get_posts", get(get_sql))
        .route("/api/submit_post", post(post_sql))
        .with_state(db_pool)
        .fallback_service(get(|req: Request<Body>| async move {
            let res = ServeDir::new(&opt.static_dir).oneshot(req).await.unwrap(); // serve dir is infallible
            let status = res.status();
            match status {
                StatusCode::NOT_FOUND => {
                    let index_path = PathBuf::from(&opt.static_dir).join("index.html");
                    fs::read_to_string(index_path).await.map_or_else(
                        |_| (StatusCode::NOT_FOUND, "index.html not found").into_response(),
                        |index_content| (StatusCode::OK, Html(index_content)).into_response(),
                    )
                }

                // path was found as a file in the static dir
                _ => res.into_response(),
            }
        }))
        .layer(ServiceBuilder::new().layer(TraceLayer::new_for_http()));

    let sock_addr = SocketAddr::from((
        IpAddr::from_str("127.0.0.1").unwrap_or(IpAddr::V6(Ipv6Addr::LOCALHOST)),
        opt.port,
    ));

    pyo3::prepare_freethreaded_python();
    tracing::debug!("python ready");

    tracing::info!("listening on http://{sock_addr}");
    tracing::info!("in directory: {:#?}", env::current_dir()?);

    axum::Server::bind(&sock_addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
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

        match chat.call_method("create", ("gpt-3.5-turbo",), Some(build_args)) {
            Ok(res) => res.to_string(),
            Err(err) => format!(
                "Error: {}\nTrace: {}",
                err.value(py),
                err.traceback(py).unwrap()
            ),
        }
    })
}

async fn get_sql(State(db_pool): State<Pool<Sqlite>>) -> Result<Json<Vec<Post>>> {
    let db_posts: Vec<DBPost> = sqlx::query_as::<_, DBPost>("SELECT * from posts")
        .fetch_all(&db_pool)
        .await?;
    let db_comments: Vec<DBComment> =
        sqlx::query_as::<_, DBComment>("SELECT post_id, username, content from comments")
            .fetch_all(&db_pool)
            .await?;

    let mut posts: Vec<Post> = Vec::with_capacity(db_posts.len());

    for db_post in db_posts {
        let comments = db_comments
            .iter()
            .filter(|c| c.post_id == db_post.id)
            .map(Comment::from_db)
            .collect::<Vec<Comment>>();

        let comments = if comments.is_empty() {
            None
        } else {
            Some(comments)
        };

        posts.push(Post::from_db(db_post, comments));
    }

    tracing::debug!("got: {:#?}", posts);

    Ok(Json(posts))
}

async fn post_sql(
    State(db_pool): State<Pool<Sqlite>>,
    Json(input): Json<Post>,
) -> impl IntoResponse {
    tracing::debug!("recieved {:?}", input);

    let last_post_id: u32 = sqlx::query("SELECT id FROM posts ORDER BY id DESC LIMIT 1")
        .fetch_one(&db_pool)
        .await
        .map_or(0, |row| row.get::<u32, usize>(0));

    let res = sqlx::query("INSERT INTO posts (id, username, content) VALUES ($1, $2, $3)")
        .bind(last_post_id + 1)
        .bind(&input.username)
        .bind(&input.content)
        .execute(&db_pool)
        .await;

    tokio::spawn(async move {
        let response: String = get_advice(&input.content);

        let last_comment_id: u32 = sqlx::query("SELECT id FROM comments ORDER BY id DESC LIMIT 1")
            .fetch_one(&db_pool)
            .await
            .map_or(0, |row| row.get::<u32, usize>(0));

        sqlx::query(
            "INSERT INTO comments (id, post_id, username, content) VALUES ($1, $2, $3, $4)",
        )
        .bind(last_comment_id + 1)
        .bind(last_post_id + 1)
        .bind("AI")
        .bind(response)
        .execute(&db_pool)
        .await
        .unwrap();

        tracing::info!("{:?}: ai done", input);
    });

    match res {
        Ok(_) => (StatusCode::ACCEPTED, "OK".to_string()),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, format!("{err}")),
    }
}
