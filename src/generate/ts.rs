//! Generates API definition in TypeScript language.

use crate::{HasMethod, IsApi};
use ts_rs::TS;

struct TsMethod {
  method_name: String,
  request_type: String,
  response_type: String,
}

pub struct TsApi {
  types: Vec<String>,
  methods: Vec<TsMethod>,
}

pub trait TraverseTsClient<API> {
  fn add_methods(ts_api: &mut TsApi);
}

impl<API> TraverseTsClient<API> for () {
  fn add_methods(_ts_api: &mut TsApi) {}
}

impl<T, N, Res, API> TraverseTsClient<API> for (T, N)
where
  T: TS,
  Res: TS,
  API: HasMethod<T, Res = Res>,
  N: TraverseTsClient<API>,
{
  fn add_methods(ts_api: &mut TsApi) {
    ts_api.methods.push(TsMethod {
      method_name: API::METHOD_NAME.into(),
      request_type: <T as TS>::inline(),
      response_type: <Res as TS>::inline(),
    });
    N::add_methods(ts_api);
  }
}

/// Generates API definition in TypeScript language.
pub fn gen_ts_api<API>() -> String
where
  API::Methods: TraverseTsClient<API>,
  API: IsApi,
{
  let mut ts_api = TsApi { types: Vec::new(), methods: Vec::new() };
  API::Methods::add_methods(&mut ts_api);
  [
    ts_api.types.join("\n"),
    "type Request<M> = ".into(),
    ts_api
      .methods
      .iter()
      .map(|method| format!("  M extends '{}' ? {} :", method.method_name, method.request_type,))
      .collect::<Vec<_>>()
      .join("\n"),
    "  void;\n".into(),
    "type Response<M> = ".into(),
    ts_api
      .methods
      .iter()
      .map(|method| format!("  M extends '{}' ? {} :", method.method_name, method.response_type,))
      .collect::<Vec<_>>()
      .join("\n"),
    "  void;".into(),
  ]
  .join("\n")
}
