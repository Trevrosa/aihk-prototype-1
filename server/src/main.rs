use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::response::Html;
use axum::{
    response::IntoResponse,
    routing::{get, post},
    Router,
};

use clap::Parser;

use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::ConnectOptions;

use std::env;
use std::net::{IpAddr, Ipv6Addr, SocketAddr};
use std::path::PathBuf;
use std::str::FromStr;

use tokio::fs;
use tower::{ServiceBuilder, ServiceExt};
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;

use crate::routes::{add_comment, get_posts, submit_post};

mod routes;

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

    let db_path: &str = if std::env::current_dir()?.ends_with("server") {
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
        .route("/api/get_posts", get(get_posts::route))
        .route("/api/submit_post", post(submit_post::route))
        .route("/api/add_comment", post(add_comment::route))
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
