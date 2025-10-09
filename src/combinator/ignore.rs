//! Erasing API level errors.

use crate::{ApiMethod, ImplsApi, ImplsApiMethod, IsApi};
use documented::DocumentedOpt;

/// Transforming return types of all methods to `()` but preserving the
/// implementor error.
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

impl<API: IsApi, B: ImplsApi<API>> ImplsApi<IgnoreRes<API>> for IgnoreRes<B> {
  type Err = B::Err;
}

impl<API: IsApi, B: ImplsApi<API> + ImplsApiMethod<API, M> + Send + Sync, M: ApiMethod<API> + Send>
  ImplsApiMethod<IgnoreRes<API>, M> for IgnoreRes<B>
where
  IgnoreRes<B>: ImplsApi<API>,
{
  async fn call_api(&self, req: M) -> Result<(), <Self as ImplsApi<IgnoreRes<API>>>::Err> {
    let _ = self.0.call_api(req).await?;
    Ok(())
  }
}

/// Transforming return types of all methods to `()` and ignoring the
/// implementor errors.
///
/// Both **API** combinator and **implementor** combinator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct IgnoreErr<A>(pub A);

impl<API: IsApi> IsApi for IgnoreErr<API> {
  type MethodList = API::MethodList;
  const NAME: &str = API::NAME;
}

impl<API: IsApi, M: ApiMethod<API>> ApiMethod<IgnoreErr<API>> for M {
  type Res = ();
  const NAME: &str = M::NAME;
}

impl<M: DocumentedOpt> DocumentedOpt for IgnoreErr<M> {
  const DOCS: Option<&str> = M::DOCS;
}

impl<API: IsApi, B: ImplsApi<API>> ImplsApi<IgnoreErr<API>> for IgnoreErr<B> {
  type Err = std::convert::Infallible;
}

impl<API, B, M> ImplsApiMethod<IgnoreErr<API>, M> for IgnoreErr<B>
where
  API: IsApi,
  M: ApiMethod<API> + Send,
  B: ImplsApi<API> + ImplsApiMethod<API, M> + Send + Sync,
{
  async fn call_api(&self, req: M) -> Result<(), std::convert::Infallible> {
    let _ = self.0.call_api(req).await;
    Ok(())
  }
}
