//! Axum router builder.

use crate::{Cons, HasMethod, ImplsMethod, Nil};
use axum::{Router, extract::Json, extract::State, routing::post};

/// Builds axum router where each method is `POST /<method_name>`, the request
/// body is expected to be a json and the result is also returned as json.
pub fn mk_axum_router<API: crate::IsApi, S>() -> Router<S>
where
  API::MethodList: MkAxumRouter<API, S>,
{
  API::MethodList::router()
}

/// API method list traversal trait for building axum router for each method.
pub trait MkAxumRouter<API, E> {
  fn router() -> Router<E>;
}

impl<
  API: HasMethod<H, Res = Res>,
  H: serde::de::DeserializeOwned + Send + 'static,
  Res: serde::Serialize + Send,
  E: ImplsMethod<API, H> + Clone + Send + Sync + 'static,
  T: MkAxumRouter<API, E>,
> MkAxumRouter<API, E> for Cons<H, T>
{
  fn router() -> Router<E> {
    T::router().route(
      &format!("/{}", API::METHOD_NAME),
      post(|State(svc): State<E>, Json(request): Json<H>| async move {
        Json(svc.call_api(request).await)
      }),
    )
  }
}

impl<API, E: Clone + Send + Sync + 'static> MkAxumRouter<API, E> for Nil {
  fn router() -> Router<E> {
    Router::new()
  }
}
