//! Wraps API method responses into `Result`.

use crate::{HasMethod, IsApi};
use core::marker::PhantomData;
use documented::DocumentedOpt;

/// Wraps API method responses into `Result`.
///
/// **API** combinator.
#[repr(transparent)]
pub struct WithErr<Err, B>(pub B, PhantomData<Err>);

impl<API: IsApi, Err> IsApi for WithErr<Err, API> {
  type Methods = API::Methods;
  const API_NAME: &str = API::API_NAME;
  const API_VERSION: &str = API::API_VERSION;
}

impl<API: IsApi + HasMethod<M>, M, Err> HasMethod<M> for WithErr<Err, API> {
  type Res = Result<API::Res, Err>;
  const METHOD_NAME: &str = API::METHOD_NAME;
  const METHOD_DOCS: Option<&str> = API::METHOD_DOCS;
}

impl<Err, API: DocumentedOpt> DocumentedOpt for WithErr<Err, API> {
  const DOCS: Option<&str> = API::DOCS;
}
