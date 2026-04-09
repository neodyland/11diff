mod diff;

use crate::diff::DiffResponse;
use anyhow::Context;
use axum::{Json, Router, response::Html, routing::get};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let addr = std::env::var("LISTEN_ADDR").unwrap_or("0.0.0.0:3000".into());
    let app = Router::new().route(
        "/",
        get(async || Html(include_str!("./index.html")))
            .post(async |Json([a, b]): Json<[String; 2]>| Json(DiffResponse::build(&a, &b))),
    );
    let listener = TcpListener::bind(&addr)
        .await
        .with_context(|| format!("failed to bind {}", addr))?;

    println!("listening on http://{}", listener.local_addr()?);

    axum::serve(listener, app)
        .await
        .context("axum server exited with error")
}
