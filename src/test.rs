#![allow(clippy::bool_assert_comparison)]
use documented::DocumentedOpt;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use ts_rs::TS;

use crate::{ImplsMethod, define_api, mk_handler};

pub type Err = String;

pub type Res<A> = Result<A, Err>;

#[derive(Clone, Serialize, Deserialize, JsonSchema, TS)]
pub struct GetA;

#[derive(Clone, Serialize, Deserialize, JsonSchema, TS)]
pub struct PostA(pub bool);

/// Some example api
#[derive(DocumentedOpt)]
pub struct SomeAPI;

define_api! { SomeAPI => {
  /// Get A
  "get_a", GetA => bool;
  // not documented on purpose
  "post_a", PostA => Res<()>;
} }

#[derive(DocumentedOpt)]
pub struct SomeAPI2;

define_api! { SomeAPI2 => {
  "get_b", GetA => bool;
} }

#[derive(Clone, Serialize, Deserialize, JsonSchema, TS)]
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

#[tokio::test]
async fn direct_api_call() {
  use crate::CallApi;
  let backend = SomeBackend::default();
  let () = backend.call_api(PostA(true)).await.unwrap();
  let new_a = backend.call_api_x::<SomeAPI2, _>(GetA).await;
  assert_eq!(new_a, true);
  assert!(backend.call_api(PostA(true)).await.is_err());
}

mod test_macro_1 {
  #![allow(dead_code)]
  struct SomeAPI;
  crate::define_api! { SomeAPI, name = "Some API" => {
    /// Get A
    "get_a", super::GetA => bool;
  } }
}

mod test_macro_2 {
  #![allow(dead_code)]
  struct SomeAPI;
  crate::define_api! { SomeAPI, version = "1.1.0" => {
    /// Get A
    "get_a", super::GetA => bool;
  } }
}

mod test_macro_3 {
  #![allow(dead_code)]
  struct SomeAPI;
  crate::define_api! { SomeAPI, name = "Some API", version = "1.1.0" => {
    /// Get A
    "get_a", super::GetA => bool;
  } }
}

mod test_macro_4 {
  #![allow(dead_code)]
  struct SomeAPI;
  crate::define_api! { SomeAPI, name = "Some API", version = "1.1.0" => {
    /// Get A
    "get_a", super::GetA => bool;
  } }
}

mod test_macro_5 {
  #![allow(dead_code)]
  struct SomeAPI;
  crate::define_api! { SomeAPI, version = "1.1.0", name = "Some API" => {
    /// Get A
    "get_a", super::GetA => bool;
  } }
}
