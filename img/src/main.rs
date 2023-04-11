#![feature(let_chains)]

mod env;
mod err;
mod img;
mod root;
mod urlmap;

use std::{net::SocketAddr, time::Duration};

use axum::{
  error_handling::HandleErrorLayer,
  http::{Request, StatusCode},
  response::Response,
  routing::get,
  BoxError, Router,
};
use env::env_default;
use time::{macros::format_description, UtcOffset};
use tower::ServiceBuilder;
use tower_http::{classify::ServerErrorsFailureClass, trace::TraceLayer};
use tracing::{error, info, info_span, Span};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use crate::{fmt::time::OffsetTime, root::root};
const TIMEOUT: u64 = 600;

#[tokio::main]
async fn main() {
  let local_time = OffsetTime::new(
    UtcOffset::from_hms(8, 0, 0).unwrap(),
    format_description!("[year][month][day] [hour][minute][second] >"),
  );

  tracing_subscriber::registry()
        .with(fmt::layer().with_timer(local_time)
            .with_level(false))
        // .with_file(true)
        // .with_line_number(true)
        .with(EnvFilter::from_default_env())
        .init();

  let addr = SocketAddr::from(([0, 0, 0, 0], env_default("PORT", 9911)));
  info!("http://{}", addr);

  // https://github.com/tokio-rs/axum/discussions/1383
  let middleware = ServiceBuilder::new()
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
    .layer(
      TraceLayer::new_for_http()
        .make_span_with(|request: &Request<_>| {
          // Log the matched route's path (with placeholders not filled in).
          // Use request.uri() or OriginalUri if you want the real path.

          let method = request.method().to_string();
          let uri = request.uri().to_string();
          info_span!("", method = method, uri = uri)
        })
        .on_request(|_request: &Request<_>, _span: &Span| {
          // You can use `_span.record("some_other_field", value)` in one of these
          // closures to attach a value to the initially empty field in the info_span
          // created above.
          info!("\n> {:?}\n", _request.headers())
        })
        .on_response(|_response: &Response, _latency: Duration, _span: &Span| {
          // ...
          info!("\n< {:?} {:?}\n", _response.headers(), _latency)
        })
        .on_failure(
          |_error: ServerErrorsFailureClass, _latency: Duration, _span: &Span| {
            error!("{:?} {:?}", _error, _span)
          },
        ),
    )
    .timeout(Duration::from_secs(TIMEOUT))
    .layer(ServiceBuilder::new())
    .into_inner();

  let router = urlmap!(Router::new());

  axum::Server::bind(&addr)
    .serve(router.layer(middleware).into_make_service())
    .await
    .unwrap();
}
