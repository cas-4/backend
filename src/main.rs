#![doc = include_str!("../README.md")]
mod audio;
mod config;
mod db;
mod errors;
mod expo;
mod graphql;
mod logger;
mod routes;
mod state;
use std::{net::SocketAddr, sync::Arc, time::Duration};

use crate::config::CONFIG;
use async_graphql::{EmptySubscription, Schema};
use axum::{
    http::{header, Method, Request},
    routing::{get, post},
    Extension, Router,
};
use errors::AppError;
use tokio::net::TcpListener;
use tower_http::{
    classify::ServerErrorsFailureClass,
    cors::{Any, CorsLayer},
    sensitive_headers::SetSensitiveHeadersLayer,
    trace::TraceLayer,
};

use tracing::Span;

/// Create the app: setup everything and returns a `Router`
async fn create_app() -> Result<Router, AppError> {
    logger::setup();
    expo::setup(CONFIG.expo_access_token.clone());
    let dbclient = db::setup().await?;

    let state = state::AppState {
        client: Arc::new(dbclient),
    };

    let schema = Schema::build(
        graphql::query::Query,
        graphql::mutation::Mutation,
        EmptySubscription,
    )
    .data(state.clone())
    .finish();

    Ok(Router::new()
        .route("/assets/sounds/:id", get(audio::show_file))
        .route(
            "/graphql",
            post(graphql::routes::graphql_handler).layer(Extension(schema.clone())),
        )
        .fallback(crate::routes::page_404)
        // Mark the `Authorization` request header as sensitive so it doesn't
        // show in logs.
        .layer(SetSensitiveHeadersLayer::new(std::iter::once(
            header::AUTHORIZATION,
        )))
        // Use a layer for `TraceLayer`
        .layer(
            TraceLayer::new_for_http()
                .on_request(|request: &Request<_>, _span: &Span| {
                    tracing::info!("{} {}", request.method(), request.uri());
                })
                .on_failure(
                    |error: ServerErrorsFailureClass, latency: Duration, _span: &Span| {
                        tracing::error!("{} | {} s", error, latency.as_secs());
                    },
                ),
        )
        .layer(
            CorsLayer::new()
                .allow_methods([Method::OPTIONS, Method::GET, Method::POST])
                .allow_headers(vec![header::CONTENT_TYPE, header::AUTHORIZATION])
                .allow_origin(Any),
        )
        .layer(Extension(state)))
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    if let Ok(app) = create_app().await {
        let host = &CONFIG.allowed_host;

        let addr = match host.parse::<SocketAddr>() {
            Ok(addr) => addr,
            Err(e) => {
                panic!("`{}` {}", host, e);
            }
        };
        tracing::info!("Listening on {}", addr);

        axum::serve(TcpListener::bind(&addr).await.unwrap(), app)
            .await
            .unwrap();
    } else {
        tracing::error!("Can't create an application!");
    }
}
