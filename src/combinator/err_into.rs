//! Map errors of APIs with [`Into`].

use crate::{HasMethod, ImplsMethod, IsApi};
use core::marker::PhantomData;
use documented::DocumentedOpt;

/// Combinator to cast errors with [`Into`]
///
/// Both **API** combinator and **implementor** combinator.
#[repr(transparent)]
pub struct ErrInto<ErrO, B>(pub B, PhantomData<ErrO>);

impl<ErrO, B> ErrInto<ErrO, B> {
  pub fn new(b: B) -> Self {
    Self(b, PhantomData)
  }
}

impl<API: IsApi, ErrO> IsApi for ErrInto<ErrO, API> {
  type Methods = API::Methods;
  const API_NAME: &str = API::API_NAME;
  const API_VERSION: &str = API::API_VERSION;
}

impl<ErrO, API: DocumentedOpt> DocumentedOpt for ErrInto<ErrO, API> {
  const DOCS: Option<&str> = API::DOCS;
}

impl<M, R, ErrI, ErrO, API> HasMethod<M> for ErrInto<ErrO, API>
where
  API: HasMethod<M, Res = Result<R, ErrI>>,
{
  type Res = Result<R, ErrO>;
  const METHOD_NAME: &str = API::METHOD_NAME;
}

impl<API, M, B, R, ErrO, ErrI> ImplsMethod<ErrInto<ErrO, API>, M> for ErrInto<ErrO, B>
where
  ErrO: From<ErrI> + Send + Sync,
  B: ImplsMethod<API, M> + Send + Sync,
  API: HasMethod<M, Res = Result<R, ErrI>>,
  M: Send + Sync,
{
  async fn call_api(&self, req: M) -> Result<R, ErrO> {
    self.0.call_api(req).await.map_err(Into::into)
  }
}

#[cfg(test)]
mod test {
  #![allow(dead_code)]
  use super::*;
  struct SomeApi;
  crate::define_api! {SomeApi => {
    "foo", Foo => Result<(), ErrI>;
    "bar", Bar => Result<(), ErrI>;
  }}

  struct Foo;
  struct Bar;
  struct ErrI;
  struct ErrO;

  impl From<ErrI> for ErrO {
    fn from(_: ErrI) -> ErrO {
      ErrO
    }
  }

  struct SomeImpl;
  impl ImplsMethod<SomeApi, Foo> for SomeImpl {
    async fn call_api(&self, _: Foo) -> Result<(), ErrI> {
      Err(ErrI)
    }
  }

  impl ImplsMethod<SomeApi, Bar> for SomeImpl {
    async fn call_api(&self, _: Bar) -> Result<(), ErrI> {
      Err(ErrI)
    }
  }

  async fn test() {
    let _: Result<_, ErrI> = SomeImpl.call_api(Foo).await;
    let _: Result<_, ErrO> = ErrInto::<ErrO, _>::new(SomeImpl).call_api(Foo).await;
    type WSomeApi = ErrInto<ErrO, SomeApi>;
  }
}
