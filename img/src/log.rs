use std::{
  fmt,
  task::{Context, Poll},
};

use axum::prelude::*;
use hyper::{Request, Response};
use tower::{BoxError, Layer, Service};

struct RecordResponsesLayer;

impl<S> Layer<S> for RecordResponsesLayer {
  type Service = RecordResponsesService<S>;

  fn layer(&self, service: S) -> Self::Service {
    RecordResponsesService { inner: service }
  }
}

struct RecordResponsesService<S> {
  inner: S,
}

impl<S, ReqBody, ResBody> Service<Request<ReqBody>> for RecordResponsesService<S>
where
  S: Service<Request<ReqBody>, Response = Response<ResBody>, Error = BoxError> + Send + 'static,
  ReqBody: Send + 'static,
  ResBody: Send + 'static,
{
  type Response = Response<ResBody>;
  type Error = S::Error;
  type Future = futures::future::BoxFuture<'static, Result<Self::Response, Self::Error>>;

  fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
    self.inner.poll_ready(cx)
  }

  fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
    let mut inner = self.inner.clone();

    Box::pin(async move {
      let response = inner.call(req).await?;

      tracing::info!("Response: {:?}", ResponseInfo(&response));

      Ok(response)
    })
  }
}

struct ResponseInfo<'a, T>(&'a Response<T>);

impl<'a, T> fmt::Debug for ResponseInfo<'a, T> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("Response")
      .field("version", &self.0.version())
      .field("status", &self.0.status())
      .field("headers", self.0.headers())
      .finish()
  }
}
