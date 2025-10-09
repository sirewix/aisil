//! Functional composition `api_g(api_h(req))`
//! `g.call_api(h.call_api(r).await?)`
//!
//! This is experimental feature and it requires more strict type definitions
//! in the client code otherwise it overflows evaluating impl requirements in unexpected places.
use crate::{ApiMethod, ImplsApi, ImplsApiMethod, IsApi};

pub struct ComposeApi<API_G, API_H>(API_G, API_H);

impl<API_G: IsApi, API_H: IsApi> IsApi for ComposeApi<API_G, API_H> {
  type MethodList = API_H::MethodList;
  const DESCRIPTION: &str = "";
  const NAME: &str = "";
}

impl<
  API_G: IsApi,
  API_H: IsApi,
  GRes,
  HReq: ApiMethod<API_H, Res = GReq>,
  GReq: ApiMethod<API_G, Res = GRes>,
  //GReq: ApiMethod<API_G>,
> ApiMethod<ComposeApi<API_G, API_H>> for HReq
{
  type Res = GRes; //<GReq as ApiMethod<API_G>>::Res;
  const DESCRIPTION: &str = "";
  const NAME: &str = "";
  fn name() -> String {
    format!("{}.{}", <GReq as ApiMethod<API_G>>::NAME, <HReq as ApiMethod<API_H>>::NAME)
  }
}

impl<API_G: IsApi, API_H: IsApi, BG: ImplsApi<API_G>, BH: ImplsApi<API_H>>
  ImplsApi<ComposeApi<API_G, API_H>> for ComposeApi<BG, BH>
{
  type Err = ComposeError<BG::Err, BH::Err>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComposeError<G, H> {
  G(G),
  H(H),
}

impl<
  API_G: IsApi,
  API_H: IsApi,
  BG: ImplsApi<API_G> + ImplsApiMethod<API_G, GReq> + Send + Sync,
  BH: ImplsApi<API_H> + ImplsApiMethod<API_H, M> + Send + Sync,
  M: ApiMethod<API_H, Res = GReq> + Send,
  GReq: ApiMethod<API_G>,
> ImplsApiMethod<ComposeApi<API_G, API_H>, M> for ComposeApi<BG, BH>
{
  async fn call_api(
    &self,
    req: M,
  ) -> Result<GReq::Res, <Self as ImplsApi<ComposeApi<API_G, API_H>>>::Err> {
    let h_res = self.1.call_api(req).await.map_err(ComposeError::H)?;
    self.0.call_api(h_res).await.map_err(ComposeError::G)
  }
}
