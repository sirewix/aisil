//! Generates API definition in TypeScript language.

use crate::{ApiMethod, Cons, IsApi, Nil};
use ts_rs::TS;

pub struct TsMethod {
  pub method_name: String,
  pub request_type: String,
  pub response_type: String,
}

pub struct TsApi {
  pub types: Vec<String>,
  pub methods: Vec<TsMethod>,
}

pub trait TraverseTsClient<API> {
  fn add_methods(ts_api: &mut TsApi);
}

impl<API> TraverseTsClient<API> for Nil {
  fn add_methods(_ts_api: &mut TsApi) {}
}

impl<T: ApiMethod<API, Res = Res> + TS, Res: TS, API, N: TraverseTsClient<API>>
  TraverseTsClient<API> for Cons<T, N>
{
  fn add_methods(ts_api: &mut TsApi) {
    ts_api.methods.push(TsMethod {
      method_name: <T as ApiMethod<API>>::NAME.into(),
      request_type: <T as TS>::inline(),
      response_type: <Res as TS>::inline(),
    });
    N::add_methods(ts_api);
  }
}

/// Generates API definition in TypeScript language.
pub fn gen_ts_api<API>() -> String
where
  API::MethodList: TraverseTsClient<API>,
  API: IsApi,
{
  let mut ts_api = TsApi { types: Vec::new(), methods: Vec::new() };
  API::MethodList::add_methods(&mut ts_api);
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
