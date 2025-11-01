//! Wraps API method responses into `Result`.

use crate::{HasMethod, IsApi};
use core::marker::PhantomData;

/// Wraps API method responses into `Result`.
///
/// **API** combinator.
pub struct WithErr<Err, B>(pub B, PhantomData<Err>);

impl<API: IsApi, Err> IsApi for WithErr<Err, API> {
  type MethodList = API::MethodList;
  const API_NAME: &str = API::API_NAME;
}

impl<API: IsApi + HasMethod<M>, M, Err> HasMethod<M> for WithErr<Err, API> {
  type Res = Result<API::Res, Err>;
  const METHOD_NAME: &str = API::METHOD_NAME;
}
