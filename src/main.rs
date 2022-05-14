#[macro_use]
extern crate lazy_static;

use std::collections::HashMap;

use axum::extract::Query;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse};
use axum::routing::get;
use axum::{Router, Server};
use tera::{Context, Tera};

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
    let router = Router::new().route("/", get(index));

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
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Internal server error: {}", err),
            )
        })
}
