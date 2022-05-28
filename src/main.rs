#[macro_use]
extern crate lazy_static;

use std::collections::HashMap;

use axum::extract::Query;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse};
use axum::routing::get;
use axum::{Router, Server};
use tera::{Context, Tera};
use tokio::signal;
use tower_http::trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer};
use tower_http::LatencyUnit;
use tracing::Level;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

mod data;

lazy_static! {
    static ref TEMPLATES: Tera = {
        let mut tera = Tera::default();
        tera.add_raw_template("index.html", include_str!("../templates/index.html"))
            .unwrap();
        tera
    };
}

#[tokio::main]
async fn main() {
    let (loki_layer, loki_task) = tracing_loki::layer(
        "http://0.0.0.0:3100".parse().unwrap(),
        vec![].into_iter().collect(),
        vec![].into_iter().collect(),
    )
    .unwrap();

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "random_episode=debug,tower_http=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .with(loki_layer)
        .init();

    tokio::spawn(loki_task);

    let trace_layer = TraceLayer::new_for_http()
        .make_span_with(DefaultMakeSpan::new().include_headers(true))
        .on_request(DefaultOnRequest::new().level(Level::INFO))
        .on_response(
            DefaultOnResponse::new()
                .level(Level::INFO)
                .latency_unit(LatencyUnit::Micros),
        );

    let router = Router::new().route("/", get(index)).layer(trace_layer);

    Server::bind(&"0.0.0.0:8080".parse().unwrap())
        .serve(router.into_make_service())
        .with_graceful_shutdown(handle_shutdown())
        .await
        .unwrap();
}

async fn index(Query(params): Query<HashMap<String, String>>) -> impl IntoResponse {
    let mut ctx = Context::new();
    ctx.insert("shows", &data::get_shows());
    ctx.insert("current_show", "");

    // If a show is selected, select a random episode.
    if let Some(show) = params.get("show") {
        if let Some(episode) = data::get_random_episode(show) {
            ctx.insert("current_show", show);
            ctx.insert("episode", &episode);
        }
    }

    TEMPLATES
        .render("index.html", &ctx)
        .map(|t| Html(t))
        .map_err(|err| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Internal server error: {}", err),
            )
        })
}

async fn handle_shutdown() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
