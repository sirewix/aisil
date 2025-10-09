#![allow(clippy::bool_assert_comparison)]
use crate::{ImplsApiMethod, define_api, mk_handler};
use documented::DocumentedOpt;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use ts_rs::TS;

pub type Err = String;

pub type Res<A> = Result<A, Err>;

#[derive(Clone, Serialize, Deserialize, JsonSchema, TS)]
pub struct GetA;

#[derive(Clone, Serialize, Deserialize, JsonSchema, TS, DocumentedOpt)]
pub struct PostA(pub bool);

/// Some example api
#[derive(DocumentedOpt)]
pub struct SomeAPI;

define_api! { SomeAPI => {
  /// Get A
  get_a, GetA => bool;
  post_a, PostA => Res<()>;
} }

#[derive(DocumentedOpt)]
pub struct SomeAPI2;

define_api! { SomeAPI2 => {
  get_b, GetA => bool;
} }

#[derive(Clone, Serialize, Deserialize, JsonSchema, TS, DocumentedOpt)]
#[allow(dead_code)]
pub struct GetB;

#[derive(Clone, Default)]
pub struct SomeBackend {
  a: Arc<Mutex<bool>>,
}

impl SomeBackend {
  pub async fn get_a(&self, _: GetA) -> bool {
    *self.a.lock().await
  }
  pub async fn post_a(&self, PostA(new_a): PostA) -> Res<()> {
    let mut a = self.a.lock().await;
    (!*a).then_some(()).ok_or("can't post `a` anymore".to_owned())?;
    *a = new_a;
    Ok(())
  }
  pub async fn get_b(&self, _: GetA) -> bool {
    true
  }
}

mk_handler! {SomeAPI, SomeBackend => {
  get_a : GetA,
  post_a : PostA,
}}

// one backend can implement multiple APIs and some Methods may be in both of
// them. in this case use .call_api_x::<SomeAPI2, _>(req) to specify which API
// to use.
mk_handler! {SomeAPI2, SomeBackend => {
  get_b : GetA,
}}

#[cfg(feature = "axum")]
pub fn router() -> ::axum::Router {
  let env = SomeBackend::default();
  crate::axum::mk_axum_router::<SomeAPI, SomeBackend>().with_state(env)
}

#[tokio::test]
async fn direct_api_call() {
  use crate::CallApi;
  let backend = SomeBackend::default();
  backend.call_api(PostA(true)).await.unwrap().unwrap();
  let new_a = backend.call_api_x::<SomeAPI2, _>(GetA).await.unwrap();
  assert_eq!(new_a, true);
  assert!(backend.call_api(PostA(true)).await.unwrap().is_err());
}

#[cfg(all(feature = "reqwest", feature = "axum"))]
#[tokio::test]
async fn axum_reqwest() {
  use crate::reqwest::ApiClient;
  use std::net::Ipv4Addr;
  use tokio::time::{Duration, sleep};

  let listener = tokio::net::TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).await.unwrap();
  let server = ::axum::serve(listener, router());

  let client: ApiClient<SomeAPI> = ApiClient::new(
    reqwest::Url::parse(&format!("http://{}/", server.local_addr().unwrap())).unwrap(),
    reqwest::Client::new(),
  )
  .unwrap();

  let server_thread = tokio::spawn(async move {
    server.await.unwrap();
  });

  sleep(Duration::from_millis(100)).await;

  client.call_api(PostA(true)).await.unwrap().unwrap();
  let new_a = client.call_api(GetA).await.unwrap();
  assert_eq!(new_a, true);
  assert!(client.call_api(PostA(true)).await.unwrap().is_err());

  server_thread.abort();
}
