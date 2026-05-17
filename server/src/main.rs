use anyhow::Context;
use axum::{
    Json, Router,
    http::{HeaderValue, header},
    response::Html,
    routing::get,
};
use iidiff_core::diff::DiffResponse;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let addr = std::env::var("LISTEN_ADDR").unwrap_or("0.0.0.0:3000".into());
    let app = Router::new()
        .route(
            "/",
            get(async || Html(include_str!("../../wasm/pkg/index.html")))
                .post(async |Json([a, b]): Json<[String; 2]>| Json(DiffResponse::build(&a, &b))),
        )
        .route(
            "/sitemap.xml",
            get(async || {
                (
                    [(
                        header::CONTENT_TYPE,
                        HeaderValue::from_static("application/xml"),
                    )],
                    include_str!("./sitemap.xml"),
                )
            }),
        )
        .route(
            "/client.js",
            get(async || {
                (
                    [(
                        header::CONTENT_TYPE,
                        HeaderValue::from_static("text/javascript"),
                    )],
                    include_str!("../../wasm/pkg/client.js"),
                )
            }),
        );
    let listener = TcpListener::bind(&addr)
        .await
        .with_context(|| format!("failed to bind {}", addr))?;

    println!("listening on http://{}", listener.local_addr()?);

    axum::serve(listener, app)
        .await
        .context("axum server exited with error")
}
