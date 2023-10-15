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

use anyhow::Result;

use clap::Parser;

use server::{AppError, Post};
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
async fn main() -> Result<()> {
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

        tracing::info!("db,table exists");
    }

    let db_pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(db_path)
        .await?;

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

    tracing::info!("listening on http://{sock_addr}");
    tracing::info!("in directory: {:#?}", env::current_dir().unwrap());

    axum::Server::bind(&sock_addr)
        .serve(app.into_make_service())
        .await
        .expect("Unable to start server");

    Ok(())
}

async fn get_sql(State(db_pool): State<Pool<Sqlite>>) -> Result<Json<Vec<Post>>, AppError> {
    let got = sqlx::query_as::<_, Post>("SELECT * from posts")
        .fetch_all(&db_pool)
        .await?;

    Ok(Json(got))
}

async fn post_sql(
    State(db_pool): State<Pool<Sqlite>>,
    Json(input): Json<Post>,
) -> impl IntoResponse {
    tracing::info!("recieved {:?}", input);

    let last = sqlx::query("SELECT id FROM posts ORDER BY id DESC LIMIT 1")
        .fetch_one(&db_pool)
        .await;

    let last: u32 = if let Ok(last) = last {
        last.get::<u32, usize>(0)
    } else {
        0
    };

    let res = sqlx::query("INSERT INTO posts (id, username, content) VALUES ($1, $2, $3)")
        .bind(last + 1)
        .bind(input.username)
        .bind(input.content)
        .execute(&db_pool)
        .await;

    match res {
        Ok(_) => (StatusCode::ACCEPTED, "OK".to_string()),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, format!("{err}")),
    }
}
