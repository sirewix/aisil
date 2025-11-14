//! Call API as HTTP `POST /<method_name>` with JSON bodies.

use crate::{HasMethod, ImplsMethod, IsApi, combinator::WithErr};
use core::marker::PhantomData;
use reqwest::{Client, Error, Url};

/// Wrapper over [`reqwest::Client`] with fixed base URL.
///
/// Calls APIs as `POST /<method_name>`
pub struct PostJsonClient<API> {
  base_url: Url,
  client: Client,
  api_marker: PhantomData<API>,
}

impl<API> Clone for PostJsonClient<API> {
  fn clone(&self) -> Self {
    PostJsonClient {
      base_url: self.base_url.clone(),
      client: self.client.clone(),
      api_marker: PhantomData,
    }
  }
}

impl<API, Req, Res> ImplsMethod<WithErr<Error, API>, Req> for PostJsonClient<API>
where
  API: IsApi + HasMethod<Req, Res = Res> + Send + Sync,
  Req: serde::Serialize + Send,
  Res: serde::de::DeserializeOwned,
{
  async fn call_api(&self, req: Req) -> Result<Res, Error> {
    self
      .client
      .post(self.base_url.join(API::METHOD_NAME).unwrap())
      .json(&req)
      .send()
      .await?
      .error_for_status()?
      .json::<Res>()
      .await
  }
}

impl<API> PostJsonClient<API> {
  pub fn new(base_url: Url, client: Client) -> Option<Self> {
    (!base_url.cannot_be_a_base()).then_some(PostJsonClient {
      base_url,
      client,
      api_marker: PhantomData,
    })
  }
}
