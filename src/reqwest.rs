//! Client to call APIs over HTTP.

use crate::{ApiMethod, ImplsApi, ImplsApiMethod, IsApi};
use reqwest::{Client, Error, Url};
use std::marker::PhantomData;

/// Wrapper over [`reqwest::Client`] with fixed base URL.
///
/// Calls APIs as `POST /<method_name>`
pub struct ApiClient<API> {
  base_url: Url,
  client: Client,
  api_marker: PhantomData<API>,
}

impl<API> Clone for ApiClient<API> {
  fn clone(&self) -> Self {
    ApiClient {
      base_url: self.base_url.clone(),
      client: self.client.clone(),
      api_marker: PhantomData,
    }
  }
}

impl<API: IsApi> ImplsApi<API> for ApiClient<API> {
  type Err = Error;
}

impl<
  API: IsApi + Send + Sync,
  Req: ApiMethod<API, Res = Res> + serde::Serialize + Send,
  Res: serde::de::DeserializeOwned,
> ImplsApiMethod<API, Req> for ApiClient<API>
{
  async fn call_api(&self, req: Req) -> Result<Res, Error> {
    self
      .client
      .post(self.base_url.join(<Req as ApiMethod<API>>::NAME).unwrap())
      .json(&req)
      .send()
      .await?
      .error_for_status()?
      .json::<Res>()
      .await
  }
}

impl<API> ApiClient<API> {
  pub fn new(base_url: Url, client: Client) -> Option<Self> {
    (!base_url.cannot_be_a_base()).then_some(ApiClient {
      base_url,
      client,
      api_marker: PhantomData,
    })
  }
}
