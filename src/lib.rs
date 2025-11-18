//! A lightweight framework to define APIs as types.
//!
//! The **core functionality** of this crate is to define APIs as types bound by
//! traits. Based on these definitions it is possible to reason about an API
//! from different perspectives: derive, type check and much more.
//!
//! - A **method** is defined as `Request` → (method name, `Response`)
//!   dependency in the context of an API (See [`HasMethod`]).
//!
//! - An **API** is defined as `ApiMetaType` → `[*]` dependency (See [`IsApi`]),
//!   where `[*]` is a heterogeneous list of request types that belong to the
//!   API.
//!
//! You can define these traits manually, but you should use [`define_api`]
//! macro instead.
//!
//! The **extra functionality** of this crate is to provide some tools to build
//! implementors of APIs and some basic middleware/combinators for those
//! implementors.
//!
//! - An **implementor** is a type that implements an API via [`ImplsMethod`].
//!   The `ImplsMethod` trait is somewhat similar to `tower::Service` and other
//!   similar traits.
//!
//! It is possible to use only the **core functionality** of this crate in
//! combination with any "service trait". But since `tower` is quite old and
//! low-level and there's no established industry standard alternatives, this
//! crate uses `ImplsMethod` to define that extra functionality.

#![cfg_attr(any(docs, docsrs), feature(doc_cfg))]

pub mod combinator;

mod json_rpc;
mod post_json;

/// Utilities for exposing an API implementor as a server.
pub mod server {
  #[cfg(feature = "json-rpc-server")]
  pub use crate::json_rpc::server as json_rpc;
  #[cfg(feature = "post-json-axum")]
  pub use crate::post_json::server as post_json;
}

/// Utilities for calling an API as a client.
#[cfg(feature = "client")]
pub mod client {
  pub use crate::json_rpc::client as json_rpc;
  pub use crate::post_json::client as post_json;
}

/// Utilities to generate specifications, IDLs, SDKs, etc.
pub mod generate;

// pub mod func;

#[doc(hidden)]
pub mod internal {
  pub use paste::paste;
}

#[cfg(test)]
mod test;

use core::future::Future;
use core::pin::Pin;

/// API definition as a type. Use [`define_api`] macro to define this impl.
pub trait IsApi {
  /// Heterogeneous list of all request types (which are also methods).
  type Methods;

  /// Name of the API. Currently is set to stringified API type name.
  const API_NAME: &str;
  const API_VERSION: &str;
}

/// Type dependency from `Request` → `Response` in the context of an API
///
/// API types must define this trait for each of their methods.
pub trait HasMethod<M>: IsApi {
  type Res;
  const METHOD_NAME: &str;
  const METHOD_DOCS: Option<&str>;
}

/// Generalization over an asyncronous function bound by an API definition.
/// Similar to `tower::Service` but associated types are defined in the API
/// definition instead of the trait itself.
pub trait ImplsMethod<API: HasMethod<M>, M> {
  fn call_api(&self, _req: M) -> impl Future<Output = API::Res> + Send;
}

/// Wrapper for implementors of [`ImplsMethodBoxed`]
///
/// ```ignore
/// type BoxedSomeAPI = BoxedImpl<Arc<dyn ImplsSomeAPI>>;
/// ```
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct BoxedImpl<B>(pub B);

use core::ops::Deref;

impl<API, M, B> ImplsMethod<API, M> for BoxedImpl<B>
where
  API: IsApi + HasMethod<M> + 'static,
  M: Send,
  B: Deref + Sync,
  B::Target: ImplsMethodBoxed<API, M>,
{
  async fn call_api(&self, req: M) -> API::Res {
    self.0.call_api_box(req).await
  }
}

// no-std feature?
impl<API, M, B> ImplsMethod<API, M> for std::sync::Arc<B>
where
  API: IsApi + HasMethod<M>,
  M: Send,
  B: ImplsMethod<API, M> + Send + Sync + ?Sized,
{
  async fn call_api(&self, req: M) -> API::Res {
    self.as_ref().call_api(req).await
  }
}

impl<API, M, B> ImplsMethod<API, M> for Box<B>
where
  API: IsApi + HasMethod<M>,
  M: Send,
  B: ImplsMethod<API, M> + Send + Sync + ?Sized,
{
  async fn call_api(&self, req: M) -> API::Res {
    self.as_ref().call_api(req).await
  }
}

/// Same as [`ImplsMethod`] but dyn-compatible
#[allow(clippy::type_complexity)]
pub trait ImplsMethodBoxed<API: HasMethod<M>, M>: Sync {
  #[must_use]
  fn call_api_box<'s, 'a>(&'s self, _req: M) -> Pin<Box<dyn Future<Output = API::Res> + Send + 'a>>
  where
    's: 'a,
    Self: 's,
    API: 'static,
    M: 'a;
}

impl<API, M, B> ImplsMethodBoxed<API, M> for B
where
  B: ImplsMethod<API, M> + Sync,
  API: HasMethod<M>,
{
  fn call_api_box<'s, 'a>(&'s self, req: M) -> Pin<Box<dyn Future<Output = API::Res> + Send + 'a>>
  where
    's: 'a,
    Self: 'a,
    API: 'a,
    M: 'a,
  {
    Box::pin(self.call_api(req))
  }
}

/// Helper trait to allow pretty type applications that [`ImplsMethod`] does
/// not allow. You may need it when one request belongs to two APIs and a
/// backend implements both of them. You should not reuse requests in different
/// APIs but in case you do, this would be helpful.
pub trait CallApi {
  fn call_api_x<API, Req>(&self, req: Req) -> impl Future<Output = API::Res> + Send
  where
    API: HasMethod<Req>,
    Self: ImplsMethod<API, Req>;
}

impl<E> CallApi for E {
  fn call_api_x<API, Req>(&self, req: Req) -> impl Future<Output = API::Res> + Send
  where
    API: HasMethod<Req>,
    Self: ImplsMethod<API, Req>,
  {
    self.call_api(req)
  }
}

#[macro_export]
#[doc(hidden)]
macro_rules! impl_method {
  {$api:ty, $method:expr, $req:ty, $res:ty, docs = $docs:expr} => {
    impl $crate::HasMethod<$req> for $api {
      type Res = $res;
      const METHOD_NAME: &str = $method;
      const METHOD_DOCS: Option<&str> = $docs;
    }
  };
  {$api:ty, $method:expr, $req:ty, $res:ty, () } => {
    $crate::impl_method!{$api, $method, $req, $res, docs = None}
  };
  {$api:ty, $method:expr, $req:ty, $res:ty, ( $($doc:expr),+ ) } => {
    $crate::impl_method!{$api, $method, $req, $res, docs = Some(concat!($($doc, "\n"),*).trim_ascii())}
  };
}

#[macro_export]
#[doc(hidden)]
macro_rules! build_hlist {
  () => { () };
  ($type:ty $(, $rest:ty)*) => { ($type, $crate::build_hlist!($($rest),*)) };
}

/// Main macro to define APIs in `aisil` format.
///
/// ```
/// pub struct SomeAPI;
///
/// aisil::define_api! { pub SomeAPI => {
///   // /// method description
///   // method_name, RequestType => ResponseType;
///
///   /// Get A
///   "get_a", GetA => bool;
///
///   // not documented method
///   "post_a", PostA => Result<(), ()>;
/// } }
///
/// pub struct GetA;
/// pub struct PostA(pub bool);
/// # fn main() {} // https://github.com/rust-lang/rust/issues/130274
/// ```
///
/// This macro generates
/// - [`IsApi`] impl for the API type.
/// - [`HasMethod`] impl for each request type.
/// - Custom `ImplsApiName` trait alias with [`ImplsMethod`] supertraits for
///   each method. Useful for dependency inversion.
/// - Custom `ImplsApiNameBoxed` trait alias with [`ImplsMethodBoxed`]
///   supertraits for each method. Useful for dependency injection.
/// - Custom `per_ApiName_method` macro in case you need to derive something
///   based on the API but cannot do that based solely on types and traits. This
///   is advanced experimental feature that you probably will not need. In case
///   you do, see the source code.
#[macro_export]
macro_rules! define_api {
  { $vis:vis $api:ident $(, version = $version:expr)? $(, name = $name:expr)? => {
    $( $( #[doc = $doc:expr] )* $method:literal, $req:ty => $res:ty; )+
  }} => {
    $crate::impl_is_api!{$vis $api, ($($req),+) $(, version = $version)? $(, name = $name)?}
    $( $crate::impl_method!{ $api, $method, $req, $res, ($($doc),*) } )+
  };

  { $vis:vis $api:ident $(, name = $name:expr)? $(, version = $version:expr)? => {
    $( $( #[doc = $doc:expr] )* $method:literal, $req:ty => $res:ty; )+
  }} => {
    $crate::impl_is_api!{$vis $api, ($($req),+) $(, version = $version)? $(, name = $name)?}
    $( $crate::impl_method!{ $api, $method, $req, $res, ($($doc),*) } )+
  };
}

#[doc(hidden)]
#[macro_export]
macro_rules! impl_is_api {
  { $vis:vis $api:ident, ($($req:ty),+), version = $version:expr, name = $name:expr } => {
      $crate::impl_is_api!{$vis $api, ($($req),+), $version, $name}
  };
  { $vis:vis $api:ident, ($($req:ty),+), version = $version:expr } => {
      $crate::impl_is_api!{$vis $api, ($($req),+), $version, stringify!($api)}
  };
  { $vis:vis $api:ident, ($($req:ty),+), name = $name:expr } => {
      $crate::impl_is_api!{$vis $api, ($($req),+), "0.0.0", $name}
  };
  { $vis:vis $api:ident, ($($req:ty),+) } => {
      $crate::impl_is_api!{$vis $api, ($($req),+), "0.0.0", stringify!($api)}
  };
  {$vis:vis $api:ident, ($($req:ty),+), $version:expr, $name:expr} => {
      impl $crate::IsApi for $api {
        type Methods = $crate::build_hlist!($($req),+);
        const API_NAME: &str = $name;
        const API_VERSION: &str = $version;
      }

      $crate::internal::paste! {
        // creating custom macro for the api to give users more extendability powers
        #[allow(unused_macros)]
        macro_rules! [<per_ $api _method>] {
          ($m:path) => { $m!{$($req),*} }
        }

        #[allow(dead_code)]
        $vis trait [<Impls $api>]: $($crate::ImplsMethod<$api, $req> +)* {}
        impl<B: $($crate::ImplsMethod<$api, $req> +)*> [<Impls $api>] for B {}

        #[allow(dead_code)]
        $vis trait [<Impls $api Boxed>]: $($crate::ImplsMethodBoxed<$api, $req> +)* {}
        impl<B: $($crate::ImplsMethodBoxed<$api, $req> +)*> [<Impls $api Boxed>] for B {}
      }
  };
}

// experimental
#[doc(hidden)]
#[macro_export]
macro_rules! mk_handler {
  ($api:ty, $envt:ty => { $($func:ident : $req:ty ,)+ } ) => (
      $(
        impl $crate::ImplsMethod<$api, $req> for $envt {
          async fn call_api(&self, req: $req) -> <$api as $crate::HasMethod<$req>>::Res
          {
            self.$func(req).await
          }
        }
      )+
  )
}
