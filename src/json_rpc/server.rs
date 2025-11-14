//! Make a server as JsonRPC.
//!
//! See [`json_rpc_router`].

use crate::{HasMethod, ImplsMethod, IsApi};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::{Value, value::RawValue};

/// User this function inside `hyper` or `axum` request handler.
pub async fn json_rpc_router<API, E>(implementor: &E, req: JsonRpcRequest) -> JsonRpcResponse
where
  API: IsApi,
  API::Methods: MkJsonRpcRouter<API, E>,
  E: Sync,
{
  let id = req.id;
  let res =
    <API::Methods as MkJsonRpcRouter<API, E>>::handle(implementor, req.method, req.params.payload)
      .await;
  match res {
    Ok(res) => JsonRpcResponse { result: Some(res), error: None, id, jsonrpc: Some("2.0") },
    Err(err) => JsonRpcResponse { result: None, error: Some(err.into()), id, jsonrpc: Some("2.0") },
  }
}

#[derive(Debug)]
pub enum JsonRpcRouterError {
  MethodNotFound(String),
  // InvalidRequest(&'static str),
  InvalidParams(serde_json::Error),
  ResponseSerialization(serde_json::Error),
}

pub trait MkJsonRpcRouter<API, E> {
  fn handle(
    implementor: &E,
    method: String,
    req: Box<RawValue>,
  ) -> impl Future<Output = Result<Value, JsonRpcRouterError>> + Send;
}

impl<API, E, H, T> MkJsonRpcRouter<API, E> for (H, T)
where
  API: HasMethod<H>,
  T: MkJsonRpcRouter<API, E>,
  H: DeserializeOwned + Send + 'static,
  API::Res: Serialize,
  E: ImplsMethod<API, H> + Sync,
{
  async fn handle(
    implementor: &E,
    method: String,
    req: Box<RawValue>,
  ) -> Result<Value, JsonRpcRouterError> {
    if method == <API as HasMethod<H>>::METHOD_NAME {
      let req: H = serde_json::from_str(req.get()).map_err(JsonRpcRouterError::InvalidParams)?;
      let res = implementor.call_api(req).await;
      serde_json::to_value(&res).map_err(JsonRpcRouterError::ResponseSerialization)
    } else {
      T::handle(implementor, method, req).await
    }
  }
}

impl<API, E: Sync> MkJsonRpcRouter<API, E> for () {
  async fn handle(_: &E, method: String, _req: Box<RawValue>) -> Result<Value, JsonRpcRouterError> {
    Err(JsonRpcRouterError::MethodNotFound(method))
  }
}

#[derive(Debug, Clone, Deserialize)]
pub struct JsonRpcRequest {
  method: String,
  params: SingleParam,
  id: Option<Box<RawValue>>,
  // jsonrpc: Option<&'a str>,
}

#[derive(Debug, Clone, Deserialize)]
struct SingleParam {
  payload: Box<RawValue>,
}

#[derive(Debug, Clone, Serialize)]
pub struct JsonRpcResponse {
  #[serde(skip_serializing_if = "Option::is_none")]
  result: Option<Value>,
  #[serde(skip_serializing_if = "Option::is_none")]
  error: Option<JsonRpcError>,
  id: Option<Box<RawValue>>,
  jsonrpc: Option<&'static str>,
}

#[derive(Clone, Debug, Serialize)]
struct JsonRpcError {
  code: i32,
  message: String,
}

impl From<JsonRpcRouterError> for JsonRpcError {
  fn from(e: JsonRpcRouterError) -> Self {
    use JsonRpcRouterError::*;
    match e {
      MethodNotFound(method) => {
        Self { code: -32601, message: format!("Method {method:?} not found") }
      }
      // InvalidRequest(msg) => Self { code: -32600, message: msg.into() },
      InvalidParams(err) => Self { code: -32602, message: err.to_string() },
      ResponseSerialization(err) => {
        Self { code: -32603, message: format!("Response json serialization error: {err}") }
      }
    }
  }
}
