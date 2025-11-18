//! Make a server as HTTP `POST /<method_name>` with JSON bodies
//!
//! See [`mk_post_json_router`]

use crate::{HasMethod, ImplsMethod, IsApi};
use axum::{Router, extract::Json, extract::State, routing::post};
use serde::{Serialize, de::DeserializeOwned};

/// Builds axum router where each method is `POST /<method_name>`, the request
/// body is expected to be a json and the result is also returned as json.
pub fn mk_post_json_router<API: crate::IsApi, S>() -> Router<S>
where
  API::Methods: MkPostJsonRouter<API, S>,
{
  API::Methods::router()
}

/// API method list traversal trait for building axum router for each method.
///
/// Use [`mk_post_json_router`].
pub trait MkPostJsonRouter<API, E> {
  fn router() -> Router<E>;
}

impl<
  API: IsApi + HasMethod<H, Res = Res>,
  H: DeserializeOwned + Send + 'static,
  Res: Serialize,
  E: ImplsMethod<API, H> + Clone + Send + Sync + 'static,
  T: MkPostJsonRouter<API, E>,
> MkPostJsonRouter<API, E> for (H, T)
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

impl<API, E: Clone + Send + Sync + 'static> MkPostJsonRouter<API, E> for () {
  fn router() -> Router<E> {
    Router::new()
  }
}
