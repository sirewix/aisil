//! Map errors of APIs with [`Into`].

use crate::{ApiMethod, ImplsApiMethod, IsApi};
use std::marker::PhantomData;

/// API-level combinator for [`ErrInto`].
pub struct ApiErrInto<ErrO, B>(pub B, PhantomData<ErrO>);

/// Map errors of API methods with [`Into`].
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct ErrInto<B>(pub B);

impl<ErrO, B> ApiErrInto<ErrO, B> {
  pub fn new(b: B) -> Self {
    Self(b, PhantomData)
  }
}

impl<API: IsApi, ErrO> IsApi for ApiErrInto<ErrO, API> {
  type MethodList = API::MethodList;
  const NAME: &str = API::NAME;
}

impl<M, R, ErrI, ErrO, API> ApiMethod<ApiErrInto<ErrO, API>> for ErrInto<M>
where
  M: ApiMethod<API, Res = Result<R, ErrI>>,
{
  type Res = Result<R, ErrO>;
  const NAME: &str = M::NAME;
}

impl<API, M, B, R, ErrO, ErrI> ImplsApiMethod<ApiErrInto<ErrO, API>, ErrInto<M>> for ErrInto<B>
where
  ErrO: From<ErrI> + Send + Sync,
  B: ImplsApiMethod<API, M> + Send + Sync,
  M: ApiMethod<API, Res = Result<R, ErrI>> + Send + Sync,
{
  async fn call_api(&self, req: ErrInto<M>) -> Result<R, ErrO> {
    self.0.call_api(req.0).await.map_err(Into::into)
  }
}
