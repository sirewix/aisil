//! Axum router builder.

use crate::{ApiMethod, Cons, ImplsApi, ImplsApiMethod, Nil};
use axum::{Router, extract::Json, extract::State, routing::post};

/// Builds axum router where each method is `POST /<method_name>`, the request
/// body is expected to be a json and the result is also returned as json.
pub fn mk_axum_router<API: crate::IsApi, S: ImplsApi<API>>() -> Router<S>
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
  API,
  H: ApiMethod<API, Res = Res> + serde::de::DeserializeOwned + Send + 'static,
  Res: serde::Serialize + Send,
  E: ImplsApiMethod<API, H> + ImplsApi<API, Err = IE> + Clone + Send + Sync + 'static,
  IE: axum::response::IntoResponse + Send,
  T: MkAxumRouter<API, E>,
> MkAxumRouter<API, E> for Cons<H, T>
{
  fn router() -> Router<E> {
    T::router().route(
      &format!("/{}", <H as ApiMethod<API>>::NAME),
      post(|State(svc): State<E>, Json(request): Json<H>| async move {
        svc.call_api(request).await.map(Json)
      }),
    )
  }
}

impl<API, E: Clone + Send + Sync + 'static> MkAxumRouter<API, E> for Nil {
  fn router() -> Router<E> {
    Router::new()
  }
}
