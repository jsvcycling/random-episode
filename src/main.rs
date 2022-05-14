#[macro_use]
extern crate lazy_static;

use std::collections::HashMap;

use axum::extract::Query;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse};
use axum::routing::get;
use axum::{Router, Server};
use tera::{Context, Tera};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod data;

lazy_static! {
    static ref TEMPLATES: Tera = match Tera::new("templates/**/*") {
        Ok(t) => t,
        Err(e) => {
            println!("Error parsing templates: {}", e);
            std::process::exit(1);
        }
    };
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "tower_http=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let router = Router::new()
        .route("/", get(index))
        .layer(
            TraceLayer::new_for_http()
        );

    Server::bind(&"0.0.0.0:8080".parse().unwrap())
        .serve(router.into_make_service())
        .await
        .unwrap();
}

async fn index(Query(params): Query<HashMap<String, String>>) -> impl IntoResponse {
    let mut ctx = Context::new();
    ctx.insert("shows", &data::get_shows());

    // If a show is selected, select a random episode.
    if let Some(show) = params.get("show") {
        if let Some(episode) = data::get_random_episode(show) {
            println!("{:?}", episode);
            ctx.insert("current_show", show);
            ctx.insert("episode", &episode);
        } else {
            ctx.insert("current_show", "");
        }
    } else {
        ctx.insert("current_show", "");
    }

    TEMPLATES
        .render("index.html", &ctx)
        .map(|t| Html(t))
        .map_err(|err| {
            println!("Error rendering template: {}", err);
            (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error")
        })
}
