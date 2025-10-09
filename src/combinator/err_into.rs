//! Map errors of api implementors with [`Into`].

use crate::{ApiMethod, ImplsApi, ImplsApiMethod};
use std::marker::PhantomData;

/// Map errors of api implementors with [`Into`].
pub struct ErrInto<ErrO, B>(pub B, PhantomData<ErrO>);

impl<ErrO, B> ErrInto<ErrO, B> {
  pub fn new(b: B) -> Self {
    Self(b, PhantomData)
  }
}

impl<API, ErrO, B> ImplsApi<API> for ErrInto<ErrO, B>
where
  ErrO: From<<B as ImplsApi<API>>::Err>,
  B: ImplsApi<API>,
{
  type Err = ErrO;
}

impl<API, M, B, ErrO> ImplsApiMethod<API, M> for ErrInto<ErrO, B>
where
  ErrO: From<<B as ImplsApi<API>>::Err> + Send + Sync,
  B: ImplsApiMethod<API, M> + Send + Sync,
  M: ApiMethod<API> + Send + Sync,
{
  async fn call_api(&self, req: M) -> Result<M::Res, ErrO> {
    self.0.call_api(req).await.map_err(Into::into)
  }
}
