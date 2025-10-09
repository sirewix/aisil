//! Spawn a new tokio task.
use crate::{ApiMethod, ImplsApi, ImplsApiMethod, IsApi};
use std::convert::Infallible;

/// Spawns a new tokio task.
///
/// Requires API methods to return `()`. You probably want to use this in
/// combination with [`super::ignore`] and [`super::tracing`] combinators.
///
/// **Implementor** combinator.
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct ForkAndForget<E>(pub E);

impl<API: IsApi, E: ImplsApi<API>> ImplsApi<API> for ForkAndForget<E> {
  type Err = Infallible;
}

impl<API: IsApi, E, Req> ImplsApiMethod<API, Req> for ForkAndForget<E>
where
  E::Err: Send + 'static,
  E: Clone + Send + Sync + 'static,
  E: ImplsApiMethod<API, Req>,
  Req: ApiMethod<API, Res = ()> + Send + 'static,
{
  async fn call_api(&self, req: Req) -> Result<(), Infallible> {
    let inner: E = self.0.clone();
    tokio::spawn(async move { inner.call_api(req).await });
    Ok(())
  }
}

#[tokio::test]
async fn test_fork_and_forget() {
  use crate::CallApi;
  use crate::combinator::IgnoreErr;
  use crate::test::{GetA, PostA, SomeAPI, SomeBackend};
  let backend = ForkAndForget(IgnoreErr(SomeBackend::default()));
  let Ok(()) = backend.call_api(PostA(true)).await;
  tokio::time::sleep(std::time::Duration::from_millis(5)).await;
  let Ok(a) = backend.0.0.call_api_x::<SomeAPI, _>(GetA).await;
  assert!(a);
}
