//! JsonRPC

#[cfg(feature = "client")]
pub mod client;
#[cfg(feature = "json-rpc-openrpc")]
pub mod openrpc;
#[cfg(feature = "json-rpc-server")]
pub mod server;

#[cfg(test)]
#[cfg(all(feature = "client", feature = "json-rpc-server"))]
mod tests {
  #![allow(clippy::bool_assert_comparison)]
  #[tokio::test]
  async fn axum_reqwest() {
    use axum::{Router, extract::Json, extract::State, routing::post};
    use std::net::Ipv4Addr;
    use tokio::time::{Duration, sleep};

    use super::client::JsonRpcClient;
    use super::*;
    use crate::ImplsMethod;
    use crate::test::*;

    let state = SomeBackend::default();
    let router = Router::new()
      .route(
        "/rpc",
        post(async move |State(svc), Json(request): Json<server::JsonRpcRequest>| {
          Json(super::server::json_rpc_router::<SomeAPI, SomeBackend>(&svc, request).await)
        }),
      )
      .with_state(state);

    let listener = tokio::net::TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).await.unwrap();
    let server = axum::serve(listener, router);

    let client: JsonRpcClient<SomeAPI> = JsonRpcClient::new(
      reqwest::Method::POST,
      reqwest::Url::parse(&format!("http://{}/rpc", server.local_addr().unwrap())).unwrap(),
      reqwest::Client::new(),
    );

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
