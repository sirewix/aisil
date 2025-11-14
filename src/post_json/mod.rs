//! HTTP `POST /<method_name>` with JSON bodies.

#[cfg(feature = "client")]
pub mod client;
#[cfg(feature = "post-json-openapi")]
pub mod openapi;
#[cfg(feature = "post-json-axum")]
pub mod server;

#[cfg(test)]
#[cfg(all(feature = "client", feature = "post-json-axum"))]
mod tests {
  #![allow(clippy::bool_assert_comparison)]
  #[tokio::test]
  async fn axum_reqwest() {
    use super::client::PostJsonClient;
    use crate::ImplsMethod;
    use crate::test::*;
    use std::net::Ipv4Addr;
    use tokio::time::{Duration, sleep};

    let env = SomeBackend::default();
    let router = super::server::mk_post_json_router::<SomeAPI, SomeBackend>().with_state(env);

    let listener = tokio::net::TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).await.unwrap();
    let server = axum::serve(listener, router);

    let client: PostJsonClient<SomeAPI> = PostJsonClient::new(
      reqwest::Url::parse(&format!("http://{}/", server.local_addr().unwrap())).unwrap(),
      reqwest::Client::new(),
    )
    .unwrap();

    let server_thread = tokio::spawn(async move {
      server.await.unwrap();
    });

    sleep(Duration::from_millis(10)).await;

    client.call_api(PostA(true)).await.unwrap().unwrap();
    let new_a = client.call_api(GetA).await.unwrap();
    assert_eq!(new_a, true);
    assert!(client.call_api(PostA(true)).await.unwrap().is_err());

    server_thread.abort();
  }
}
