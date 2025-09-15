use std::net::SocketAddr;

use axum::{routing::get, Router};
use std::sync::Arc;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{fmt, EnvFilter};

mod api;
mod config;
mod model;
mod store;
mod web;

#[tokio::main]
async fn main() {
    // init logging
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,axum=info,hyper=info"));
    fmt().with_env_filter(env_filter).init();

    let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(3000u16);
    let addr: SocketAddr = format!("{}:{}", host, port).parse().expect("invalid addr");
    let base_url = format!("http://{}:{}", host, port);
    // base router: health check + web routes + api routes
    let health = Router::new().route("/healthz", get(|| async { "ok" }));
    let cfg = config::Config::from_env();
    let _ = cfg.ensure_exists();
    let store = Arc::new(store::Store::new(cfg.calib_root.clone()));
    let read_only = std::env::var("READ_ONLY").map(|v| matches!(&*v.to_lowercase(), "1" | "true" | "yes")).unwrap_or(false);
    let state = api::AppState { store, base_url: Some(base_url), read_only };
    let app = health
        .merge(api::router(state.clone()))
        .merge(web::router(state))
        .layer(TraceLayer::new_for_http());

    tracing::info!(%addr, "starting server");
    let listener = tokio::net::TcpListener::bind(addr).await.expect("bind");
    axum::serve(listener, app).await.expect("server error");
}
