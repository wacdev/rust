use axum::{
  http::StatusCode,
  response::{IntoResponse, Response},
};
use tracing::{error, instrument};

#[derive(Debug)]
pub struct Err(anyhow::Error);
pub type Result<T, E = Err> = anyhow::Result<T, E>;

// Tell axum how to convert `Err` into a response.
impl IntoResponse for Err {
  fn into_response(self) -> Response {
    error!("{}", self.0);
    (
      StatusCode::INTERNAL_SERVER_ERROR,
      format!("ERR: {}", self.0),
    )
      .into_response()
  }
}

// This enables using `?` on functions that return `Result<_, anyhow::Error>` to turn them into
// `Result<_, Err>`. That way you don't need to do that manually.
impl<E> From<E> for Err
where
  E: Into<anyhow::Error>,
{
  fn from(err: E) -> Self {
    Self(err.into())
  }
}
