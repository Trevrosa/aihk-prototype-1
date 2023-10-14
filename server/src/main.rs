use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::response::Html;
use axum::{
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use axum::{Extension, Json};

use clap::Parser;
use serde::{Deserialize, Serialize};
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::{ConnectOptions, Connection, FromRow};

use std::env;
use std::net::{IpAddr, Ipv6Addr, SocketAddr};
use std::path::PathBuf;
use std::str::FromStr;

use tokio::fs;

use tower::{ServiceBuilder, ServiceExt};
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;

#[allow(clippy::unused_async)]
// Setup the command line interface with clap.
#[derive(Parser, Debug)]
#[clap(name = "server", about = "Server")]
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
async fn main() {
    let opt = Opt::parse();

    // Setup logging & RUST_LOG from args
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", format!("{},hyper=info,mio=info", opt.log_level));
    }

    // enable console logging
    tracing_subscriber::fmt::init();

    let db_path = if std::env::current_dir().unwrap().ends_with("server") {
        "all.db"
    } else {
        "server/all.db"
    };

    let db_opts = SqliteConnectOptions::new()
        .filename(db_path)
        .create_if_missing(true);

    let app = Router::new()
        .route("/api/hi", get(test))
        .route("/api/get_posts", get(get_sql))
        .route("/api/submit_post", post(post_sql))
        .layer(Extension(db_opts))
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

    let db_opts = SqliteConnectOptions::new()
        .filename(db_path)
        .create_if_missing(true);

    let mut sqlite_connection = db_opts.connect().await.unwrap();
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS posts (username TEXT PRIMARY KEY, content TEXT NOT NULL)",
    )
    .execute(&mut sqlite_connection)
    .await
    .unwrap();
    pyo3::prepare_freethreaded_python();
    sqlite_connection.close();

    tracing::info!("listening on http://{sock_addr}");
    tracing::info!("in directory: {:#?}", env::current_dir().unwrap());

    axum::Server::bind(&sock_addr)
        .serve(app.into_make_service())
        .await
        .expect("Unable to start server");
}

#[derive(Debug, Deserialize, Serialize, FromRow)]
struct Post {
    username: String,
    content: String,
}

async fn get_sql(Extension(db_opts): Extension<SqliteConnectOptions>) -> Json<Vec<Post>> {
    let mut sqlite_connection = db_opts.connect().await.unwrap();
    let got = sqlx::query_as::<_, Post>("SELECT * from posts")
        .fetch_all(&mut sqlite_connection)
        .await
        .unwrap();

    Json(got)
}

async fn post_sql(
    Extension(db_opts): Extension<SqliteConnectOptions>,
    Json(input): Json<Post>,
) -> impl IntoResponse {
    tracing::info!("{:?}", input);

    let mut sqlite_connection = db_opts.connect().await.unwrap();

    let res = sqlx::query("INSERT INTO posts (username, content) VALUES ($1, $2)")
        .bind(input.username)
        .bind(input.content)
        .execute(&mut sqlite_connection)
        .await;
    match res {
        Ok(_) => StatusCode::ACCEPTED,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

async fn test() -> impl IntoResponse {
    "hi!"
}
