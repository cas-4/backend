mod config;
mod db;
mod errors;
mod graphql;
mod logger;
mod routes;
mod state;
use std::{net::SocketAddr, sync::Arc, time::Duration};

use crate::config::CONFIG;
use async_graphql::{EmptyMutation, EmptySubscription, Schema};
use axum::{
    http::{header, Request},
    routing::post,
    Extension, Router,
};
use tokio::net::TcpListener;
use tower_http::{
    classify::ServerErrorsFailureClass, sensitive_headers::SetSensitiveHeadersLayer,
    trace::TraceLayer,
};

use tracing::Span;

/// Create the app: setup everything and returns a `Router`
async fn create_app() -> Router {
    logger::setup();
    let dbclient = db::setup().await.unwrap();
    let state = state::AppState {
        client: Arc::new(dbclient),
    };

    let schema = Schema::build(graphql::query::Query, EmptyMutation, EmptySubscription)
        .data(state.clone())
        .finish();
    Router::new()
        .route(
            "/graphql",
            post(move |req| graphql::routes::graphql_handler(schema.clone().into(), req)),
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
        .layer(Extension(state))
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let app = create_app().await;

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
}