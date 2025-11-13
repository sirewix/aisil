//! Erasing API level errors.

use crate::{HasMethod, ImplsMethod, IsApi};
use documented::DocumentedOpt;

/// Transforming return types of all methods that must be `Result<R, E>` to
/// `Result<(), E>`, ignoring the result in `Ok`, but preserving the `Err`.
///
/// Both **API** combinator and **implementor** combinator.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct IgnoreOk<A>(pub A);

impl<API: IsApi> IsApi for IgnoreOk<API> {
  type MethodList = API::MethodList;
  const API_NAME: &str = API::API_NAME;
  const API_VERSION: &str = API::API_VERSION;
}

impl<API: IsApi, R, E, M> HasMethod<M> for IgnoreOk<API>
where
  API: HasMethod<M, Res = Result<R, E>>,
{
  type Res = Result<(), E>;
  const METHOD_NAME: &str = API::METHOD_NAME;
}

impl<API: DocumentedOpt> DocumentedOpt for IgnoreOk<API> {
  const DOCS: Option<&str> = API::DOCS;
}

impl<API, B, E, M, R> ImplsMethod<IgnoreOk<API>, M> for IgnoreOk<B>
where
  API: IsApi,
  B: ImplsMethod<API, M> + Send + Sync,
  API: HasMethod<M, Res = Result<R, E>>,
  M: Send,
{
  async fn call_api(&self, req: M) -> Result<(), E> {
    let _ = self.0.call_api(req).await?;
    Ok(())
  }
}

/// Transforming return types of all methods to `()`.
///
/// Both **API** combinator and **implementor** combinator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct IgnoreRes<A>(pub A);

impl<API: IsApi> IsApi for IgnoreRes<API> {
  type MethodList = API::MethodList;
  const API_NAME: &str = API::API_NAME;
  const API_VERSION: &str = API::API_VERSION;
}

impl<API: IsApi + HasMethod<M>, M> HasMethod<M> for IgnoreRes<API> {
  type Res = ();
  const METHOD_NAME: &str = API::METHOD_NAME;
}

impl<API: DocumentedOpt> DocumentedOpt for IgnoreRes<API> {
  const DOCS: Option<&str> = API::DOCS;
}

impl<API, B, M> ImplsMethod<IgnoreRes<API>, M> for IgnoreRes<B>
where
  API: IsApi + HasMethod<M>,
  M: Send,
  B: ImplsMethod<API, M> + Send + Sync,
{
  async fn call_api(&self, req: M) {
    let _ = self.0.call_api(req).await;
  }
}
