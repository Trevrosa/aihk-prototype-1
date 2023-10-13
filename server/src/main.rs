use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::response::Html;
use axum::{response::IntoResponse, routing::get, Router};

use clap::Parser;

use std::net::{IpAddr, Ipv6Addr, SocketAddr};
use std::path::PathBuf;
use std::str::FromStr;

use tokio::fs;

use tower::{ServiceBuilder, ServiceExt};
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;

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
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", format!("{},hyper=info,mio=info", opt.log_level));
    }
    // enable console logging
    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/api/hello", get(hello))
        .route("/api/py", get(python))
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

    tracing::info!("listening on http://{}", sock_addr);
    tracing::info!("in directory: {:#?}", std::env::current_dir().unwrap());

    axum::Server::bind(&sock_addr)
        .serve(app.into_make_service())
        .await
        .expect("Unable to start server");
}

async fn hello() -> impl IntoResponse {
    format!("hello from server! (at {})", humantime::format_rfc3339_millis(std::time::SystemTime::now().checked_add(std::time::Duration::from_secs(60*60*8)).unwrap()))
}

async fn python() -> impl IntoResponse {
    let out = std::process::Command::new("python").args(&["server/test.py"]).output();
    match out {
        Ok(out) => std::str::from_utf8(&out.stdout).unwrap().to_owned(),
        Err(err) => err.to_string()
    }
}
