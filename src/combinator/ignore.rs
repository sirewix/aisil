//! Erasing API level errors.

use crate::{ApiMethod, ImplsApiMethod, IsApi};
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
  const NAME: &str = API::NAME;
}

impl<API: IsApi, R, E, M> ApiMethod<IgnoreOk<API>> for IgnoreOk<M>
where
  M: ApiMethod<API, Res = Result<R, E>>,
{
  type Res = Result<(), E>;
  const NAME: &str = M::NAME;
}

impl<M: DocumentedOpt> DocumentedOpt for IgnoreOk<M> {
  const DOCS: Option<&str> = M::DOCS;
}

impl<API, B, E, M, R> ImplsApiMethod<IgnoreOk<API>, IgnoreOk<M>> for IgnoreOk<B>
where
  API: IsApi,
  B: ImplsApiMethod<API, M> + Send + Sync,
  M: ApiMethod<API, Res = Result<R, E>> + Send,
{
  async fn call_api(&self, req: IgnoreOk<M>) -> Result<(), E> {
    let _ = self.0.call_api(req.0).await?;
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
  const NAME: &str = API::NAME;
}

impl<API: IsApi, M: ApiMethod<API>> ApiMethod<IgnoreRes<API>> for M {
  type Res = ();
  const NAME: &str = M::NAME;
}

impl<M: DocumentedOpt> DocumentedOpt for IgnoreRes<M> {
  const DOCS: Option<&str> = M::DOCS;
}

impl<API, B, M> ImplsApiMethod<IgnoreRes<API>, M> for IgnoreRes<B>
where
  API: IsApi,
  M: ApiMethod<API> + Send,
  B: ImplsApiMethod<API, M> + Send + Sync,
{
  async fn call_api(&self, req: M) {
    let _ = self.0.call_api(req).await;
  }
}
