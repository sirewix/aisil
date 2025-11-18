#![allow(non_camel_case_types)]
//! Functional composition `api_g(api_h(req))`
//! `g.call_api(h.call_api(r).await)`
//!
//! This is experimental feature implemented because it is possible to implement
//! rather than a practical need.

use crate::{HasMethod, ImplsMethod, IsApi};
use documented::DocumentedOpt;

// TODO: make some use of it

/// Functional composition `api_g(api_h(req))`
/// `g.call_api(h.call_api(r).await)`
///
/// The resulting set of methods for the composed api is based on `H`.
///
/// Both **API** combinator and **implementor** combinator.
pub struct Compose<API_G, API_H>(API_G, API_H);

impl<API_G: IsApi, API_H: IsApi> IsApi for Compose<API_G, API_H> {
  type Methods = API_H::Methods;
  const API_NAME: &str = API_H::API_NAME;
  const API_VERSION: &str = API_H::API_VERSION; // not really good
}

impl<API_G, API_H: DocumentedOpt> DocumentedOpt for Compose<API_G, API_H> {
  const DOCS: Option<&str> = API_H::DOCS;
}

impl<API_G, API_H, HReq> HasMethod<HReq> for Compose<API_G, API_H>
where
  API_H: HasMethod<HReq>,
  API_G: HasMethod<API_H::Res>,
{
  type Res = API_G::Res;
  const METHOD_NAME: &str = API_H::METHOD_NAME;
  const METHOD_DOCS: Option<&str> = API_H::METHOD_DOCS;
}

impl<
  API_G: IsApi + HasMethod<GReq, Res = GRes>,
  API_H: IsApi + HasMethod<HReq, Res = GReq>,
  GReq,
  HReq: Send,
  GRes,
  BG: ImplsMethod<API_G, GReq> + Send + Sync,
  BH: ImplsMethod<API_H, HReq> + Send + Sync,
> ImplsMethod<Compose<API_G, API_H>, HReq> for Compose<BG, BH>
{
  async fn call_api(&self, req: HReq) -> GRes {
    self.0.call_api(self.1.call_api(req).await).await
  }
}

/// Similar to [`Compose`] but `API_H` methods must return `Result`s
pub struct ComposeRes<API_G, API_H>(API_G, API_H);

impl<API_G: IsApi, API_H: IsApi> IsApi for ComposeRes<API_G, API_H> {
  type Methods = API_H::Methods;
  const API_NAME: &str = API_H::API_NAME;
  const API_VERSION: &str = API_H::API_VERSION; // not really good
}

impl<API_G, API_H: DocumentedOpt> DocumentedOpt for ComposeRes<API_G, API_H> {
  const DOCS: Option<&str> = API_H::DOCS;
}

impl<API_G, API_H, HReq, GReq, HErr> HasMethod<HReq> for ComposeRes<API_G, API_H>
where
  API_H: HasMethod<HReq, Res = Result<GReq, HErr>>,
  API_G: HasMethod<GReq>,
{
  type Res = Result<API_G::Res, HErr>;
  const METHOD_NAME: &str = API_H::METHOD_NAME;
  const METHOD_DOCS: Option<&str> = API_H::METHOD_DOCS;
}

impl<
  API_G: IsApi + HasMethod<GReq, Res = GRes>,
  API_H: IsApi + HasMethod<HReq, Res = Result<GReq, HErr>>,
  GReq: Send,
  HReq: Send,
  HErr: Send,
  GRes,
  BG: ImplsMethod<API_G, GReq> + Send + Sync,
  BH: ImplsMethod<API_H, HReq> + Send + Sync,
> ImplsMethod<ComposeRes<API_G, API_H>, HReq> for ComposeRes<BG, BH>
{
  async fn call_api(&self, req: HReq) -> Result<GRes, HErr> {
    Ok(self.0.call_api(self.1.call_api(req).await?).await)
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::test::*;
  use crate::*;
  struct GApi;

  define_api! {GApi => {
    "foo", Res<()> => bool;
  }}

  struct SomeG;
  impl ImplsMethod<GApi, Res<()>> for SomeG {
    async fn call_api(&self, req: Res<()>) -> bool {
      req.is_ok()
    }
  }

  //type ComposedApi = Compose<GApi, SomeAPI>;
  type ComposedImplementor = Compose<SomeG, SomeBackend>;

  #[tokio::test]
  async fn test() {
    let backend: ComposedImplementor = Compose(SomeG, SomeBackend::default());
    assert!(backend.call_api(PostA(true)).await);
    assert!(!backend.call_api(PostA(true)).await);
  }
}

// TODO: ComposeRes where API_H methods return results
