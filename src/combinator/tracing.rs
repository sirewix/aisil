//! Simple tracing combinator.

use crate::{ApiMethod, ImplsApi, ImplsApiMethod, IsApi};
use std::fmt::Debug;

/// Simple tracing combinator.
///
/// Adds a span and can optionally trace requests, responses and errors.
///
/// **Implementor** combinator.
#[derive(Debug, Clone, Copy)]
pub struct ApiTracer<E>(pub ApiTracerConfig, pub E);

/// Parameters to [`ApiTracer`]. By default only logs errors.
#[derive(Debug, Clone, Copy)]
pub struct ApiTracerConfig {
  /// Enable `debug!("Request: {req:?}")`, default: `false`.
  pub request: bool, // TODO: tracing::level_filters::LevelFilter or similar
  /// Enable `debug!("Response: {res:?}")`, default: `false`.
  pub response: bool,
  /// Enable `error!("{err:?}")`, default: `true`.
  pub error: bool,
}

impl Default for ApiTracerConfig {
  fn default() -> Self {
    Self { request: false, response: false, error: true }
  }
}

impl<API, E: ImplsApi<API>> ImplsApi<API> for ApiTracer<E> {
  type Err = E::Err;
}

impl<API, E, Req, Res> ImplsApiMethod<API, Req> for ApiTracer<E>
where
  E::Err: Debug, // TODO: use Display?
  E: Send + Sync,
  E: ImplsApiMethod<API, Req>,
  Req: ApiMethod<API, Res = Res> + Send + Debug,
  Res: Debug,
  API: IsApi,
{
  #[rustfmt::skip]
  #[tracing::instrument(skip_all, fields(API = API::NAME, method = <Req as ApiMethod<API>>::NAME))]
  async fn call_api(&self, req: Req) -> Result<Res, E::Err> {
    let when = &self.0;
    if when.request { tracing::debug!("Request: {req:?}") }
    self.1.call_api(req).await
      .inspect(|r| if when.response {tracing::debug!("Response: {r:?}")})
      .inspect_err(|e| if when.error {tracing::error!("{e:?}")})
  }
}
