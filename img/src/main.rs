#![feature(let_chains)]

mod env;
mod err;
mod img;
mod root;
mod urlmap;

use std::{net::SocketAddr, time::Duration};

use axum::{error_handling::HandleErrorLayer, http::StatusCode, routing::get, BoxError, Router};
use env::env_default;
use time::macros::format_description;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{
  fmt, fmt::time::UtcTime, prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt,
  EnvFilter,
};

use crate::root::root;

const TIMEOUT: u64 = 600;

#[tokio::main]
async fn main() {
  let timer = UtcTime::new(format_description!(
    "[year][month][day]>[hour]:[minute]:[second]"
  ));

  tracing_subscriber::registry()
    .with(fmt::layer().with_timer(timer))
    .with(EnvFilter::from_default_env())
    .init();

  let addr = SocketAddr::from(([0, 0, 0, 0], env_default("PORT", 9911)));
  tracing::info!("http://{}", addr);

  // https://github.com/tokio-rs/axum/discussions/1383
  let middleware = ServiceBuilder::new()
    .layer(ServiceBuilder::new().layer(TraceLayer::new_for_http()))
    .layer(HandleErrorLayer::new(|error: BoxError| async move {
      if error.is::<tower::timeout::error::Elapsed>() {
        Ok(StatusCode::REQUEST_TIMEOUT)
      } else {
        Err((
          StatusCode::INTERNAL_SERVER_ERROR,
          format!("Unhandled internal error: {}", error),
        ))
      }
    }))
    .timeout(Duration::from_secs(TIMEOUT))
    .into_inner();

  let router = urlmap!(Router::new());

  axum::Server::bind(&addr)
    .serve(router.layer(middleware).into_make_service())
    .await
    .unwrap();
}
