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
//!   similar traits. It is possible to use only the **core functionality** of
//!   this crate in combination with any "service trait". But since `tower` is
//!   quite old and low-level and there's no established industry standards
//!   alternatives, this crate uses `ImplsMethod` to define the extra
//!   functionality.

#![cfg_attr(any(docs, docsrs), feature(doc_cfg))]

use core::future::Future;
use core::pin::Pin;

#[cfg(feature = "axum")]
pub mod axum;
pub mod combinator;
#[cfg(feature = "openapi")]
pub mod openapi;
#[cfg(feature = "reqwest")]
pub mod reqwest;
#[cfg(feature = "ts")]
pub mod ts;
// pub mod func;

#[doc(hidden)]
pub mod internal {
  pub use documented::DocumentedOpt;
  pub use paste::paste;
  pub use trait_set::trait_set;
}

#[cfg(test)]
mod test;

/// API definition as a type. Use [`define_api`] macro to define this impl.
pub trait IsApi {
  /// Heterogeneous list of all request types (which are also methods).
  type MethodList;
  /// Name of the API. Currently is set to stringified API type name.
  const API_NAME: &str;
}

pub trait HasMethod<M>: IsApi {
  type Res;
  const METHOD_NAME: &str;
}

/// Generalization over an asyncronous function bound by an API definition.
/// Similar to `tower::Service` but associated types are defined in the API
/// definition instead of the trait itself.
pub trait ImplsMethod<API: HasMethod<M>, M> {
  fn call_api(&self, _req: M) -> impl Future<Output = API::Res> + Send;
}

impl<API, M, B> ImplsMethod<API, M> for Box<B>
where
  API: IsApi + HasMethod<M>,
  M: Send,
  B: ImplsMethod<API, M> + ?Sized,
{
  fn call_api(&self, req: M) -> impl Future<Output = API::Res> + Send {
    self.as_ref().call_api(req)
  }
}

impl<API, M, B> ImplsMethod<API, M> for std::sync::Arc<B>
where
  API: IsApi + HasMethod<M>,
  M: Send,
  B: ImplsMethod<API, M> + ?Sized,
{
  fn call_api(&self, req: M) -> impl Future<Output = API::Res> + Send {
    self.as_ref().call_api(req)
  }
}

/// Same as [`ImplsMethod`] but dyn-compatible
#[allow(clippy::type_complexity)]
pub trait ImplsMethodBoxed<API: HasMethod<M>, M>: Sync {
  #[must_use]
  fn call_api_box<'s, 'a>(&'s self, _req: M) -> Pin<Box<dyn Future<Output = API::Res> + Send + 'a>>
  where
    's: 'a,
    Self: 'a,
    API: 'a,
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

// use frunk::HList maybe?
pub struct Cons<T, N>(T, N);
pub struct Nil;

#[macro_export]
#[doc(hidden)]
macro_rules! def_method {
  {$api:ty, $method:ident, $req:ty, $res:ty} => {
  }
}

#[macro_export]
#[doc(hidden)]
macro_rules! build_hlist {
  () => { $crate::Nil };
  ($type:ty $(, $rest:ty)*) => { $crate::Cons<$type, $crate::build_hlist!($($rest),*)> };
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
///   get_a, GetA => bool;
///
///   // not documented method
///   post_a, PostA => Result<(), ()>;
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
  { $vis:vis $api:ident => { $(
    $( #[doc = $doc0:expr] $( #[doc = $doc:expr] )* )?
    $method:ident, $req:ty => $res:ty;
  )+ }} => {
      impl $crate::IsApi for $api {
        type MethodList = $crate::build_hlist!($($req),+);
        const API_NAME: &str = stringify!($api);
      }

      // per method HasMethod impls
      $(
        impl $crate::HasMethod<$req> for $api {
          type Res = $res;
          const METHOD_NAME: &str = stringify!($method);
        }
        $(
          impl $crate::internal::DocumentedOpt for $req {
            const DOCS: Option<&str> = Some(concat!($doc0, $($doc, "\n"),*).trim_ascii());
          }
        )?
      )+

      $crate::internal::paste! {
        // creating custom macro for the api to give users unlimited extendability powers
        #[allow(unused_macros)]
        macro_rules! [<per_ $api _method>] {
          ($m:path) => {
            $m!{$vis $api, $(($method, $req)),*}
          }
        }

        // the module trick here because there's no easy way to put #[allow(dead_code)]
        // on trait aliases with `trait_set`
        #[doc(hidden)]
        #[allow(non_snake_case)]
        $vis mod [<$api _default_trait_aliases>] {
          #![allow(dead_code)]
          use super::*;
          // using that custom macro to derive two default trait aliases
          [<per_ $api _method>]!($crate::mk_impls_api_method_trait_alias);
          [<per_ $api _method>]!($crate::mk_impls_api_method_boxed_trait_alias);
        }

        #[allow(unused_imports)]
        $vis use [<$api _default_trait_aliases>]::*;
      }
  };
}

#[macro_export]
#[doc(hidden)]
macro_rules! mk_impls_api_method_trait_alias {
	($vis:vis $api:ty, $(($n:ident, $t:ty)),*) => {
		$crate::internal::paste! {
			$crate::internal::trait_set! {
        /// Trait alias for [`aisil::ImplsMethod`] for all methods of [`$api`]
				$vis trait [<Impls $api>] = $($crate::ImplsMethod<$api, $t> + )*;
			}
		}
	}
}

#[macro_export]
#[doc(hidden)]
macro_rules! mk_impls_api_method_boxed_trait_alias {
	($vis:vis $api:ty, $(($n:ident, $t:ty)),*) => {
		$crate::internal::paste! {
			$crate::internal::trait_set! {
        /// Trait alias for [`aisil::ImplsMethodBoxed`] for all methods of [`$api`]
				$vis trait [<Impls $api Boxed>] = $($crate::ImplsMethodBoxed<$api, $t> + )*;
			}
		}
	}
}

// TODO: add mk_impls_api_method_tower_trait_alias for tower::Service

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
