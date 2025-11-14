//! Call API as JsonRPC.

use crate::{HasMethod, ImplsMethod, IsApi, combinator::WithErr};
use core::marker::PhantomData;
use reqwest::{Client, Error, Method, Url};
use serde::{Deserialize, Serialize};

/// Wrapper over [`reqwest::Client`] with fixed base URL.
pub struct JsonRpcClient<API> {
  method: Method,
  base_url: Url,
  client: Client,
  api_marker: PhantomData<API>,
}

impl<API> Clone for JsonRpcClient<API> {
  fn clone(&self) -> Self {
    JsonRpcClient {
      method: self.method.clone(),
      base_url: self.base_url.clone(),
      client: self.client.clone(),
      api_marker: PhantomData,
    }
  }
}

impl<API, Req, Res> ImplsMethod<WithErr<Error, API>, Req> for JsonRpcClient<API>
where
  API: IsApi + HasMethod<Req, Res = Res> + Send + Sync,
  Req: serde::Serialize + Send,
  Res: serde::de::DeserializeOwned,
{
  async fn call_api(&self, req: Req) -> Result<Res, Error> {
    Ok(
      self
        .client
        .request(self.method.clone(), self.base_url.clone())
        .json(&JsonRpcRequest {
          method: API::METHOD_NAME,
          params: SingleParam { payload: req },
          jsonrpc: "2.0",
        })
        .send()
        .await?
        .error_for_status()?
        .json::<JsonRpcResponse<Res>>()
        .await?
        .result,
    )
  }
}

impl<API> JsonRpcClient<API> {
  pub fn new(method: Method, base_url: Url, client: Client) -> Self {
    Self { method, base_url, client, api_marker: PhantomData }
  }
}

#[derive(Debug, Clone, Serialize)]
struct SingleParam<P> {
  payload: P,
}

#[derive(Debug, Clone, Serialize)]
struct JsonRpcRequest<'a, P> {
  method: &'a str,
  params: SingleParam<P>,
  jsonrpc: &'static str,
}

#[derive(Debug, Clone, Deserialize)]
struct JsonRpcResponse<X> {
  result: X,
}
