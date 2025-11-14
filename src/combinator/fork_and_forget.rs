//! Spawn a new tokio task.
use crate::{HasMethod, ImplsMethod, IsApi};

/// Spawns a new tokio task.
///
/// Requires API methods to return `()`. You probably want to use this in
/// combination with [`super::IgnoreRes`] and [`super::tracing`] combinators.
///
/// **Implementor** combinator.
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct ForkAndForget<B>(pub B);

impl<API: IsApi, B, Req> ImplsMethod<API, Req> for ForkAndForget<B>
where
  B: Clone + Send + Sync + 'static,
  B: ImplsMethod<API, Req>,
  API: HasMethod<Req, Res = ()>,
  Req: Send + 'static,
{
  async fn call_api(&self, req: Req) {
    let inner: B = self.0.clone();
    tokio::spawn(async move { inner.call_api(req).await });
  }
}

#[cfg(test)]
mod test {
  #[tokio::test]
  async fn fork_and_forget() {
    use super::*;
    use crate::CallApi;
    use crate::combinator::IgnoreRes;
    use crate::test::{GetA, PostA, SomeAPI, SomeBackend};
    let backend = ForkAndForget(IgnoreRes(SomeBackend::default()));
    let () = backend.call_api(PostA(true)).await;
    tokio::time::sleep(std::time::Duration::from_millis(5)).await;
    let () = backend.call_api_x::<IgnoreRes<SomeAPI>, _>(GetA).await;
    let new_a = backend.0.0.call_api_x::<SomeAPI, _>(GetA).await;
    assert!(new_a);
  }
}
