//! Wraps API method responses into `Result`.

use crate::ApiMethod;
use core::marker::PhantomData;

/// Wraps API method responses into `Result`.
///
/// **API** combinator.
pub struct WithErr<Err, B>(pub B, PhantomData<Err>);

impl<API, M: ApiMethod<API>, Err> ApiMethod<WithErr<Err, API>> for M {
  type Res = Result<M::Res, Err>;
  const NAME: &str = M::NAME;
}
