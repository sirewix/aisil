use super::*;
use std::sync::Arc;
use tokio::sync::Mutex;
use serde::{Serialize, Deserialize};
use schemars::JsonSchema;
use ts_rs::TS;

struct SomeAPI;

type Err = i32;

type Res<A> = Result<A, Err>;

#[derive(Clone, Serialize, Deserialize, JsonSchema, TS)]
struct GetA;

#[derive(Clone, Serialize, Deserialize, JsonSchema, TS)]
struct PostA(bool);

define_api! { SomeAPI, "Some api", "Some example api" => Err {
  "get_a", GetA => bool : "Get A";
  "post_a", PostA => () : "Post A";
} }

#[cfg(feature = "axum")]
#[derive(Clone)]
struct SomeBackend {
  a: Arc<Mutex<bool>>,
}

#[cfg(feature = "axum")]
impl SomeBackend {
  pub async fn get_a(&self, _: GetA) -> Res<bool> {
    Ok(self.a.lock().await.clone())
  }
  pub async fn post_a(&self, PostA(new_a): PostA) -> Res<()> {
    let mut a = self.a.lock().await;
    *a = new_a;
    Ok(())
  }
}

#[cfg(feature = "axum")]
pub fn router() -> axum::Router {
  let env = SomeBackend {
    a: Arc::new(Mutex::new(false)),
  };
  mk_axum_router!(SomeAPI, env, SomeBackend => {
    get_a : GetA,
    post_a : PostA,
  })
}

#[cfg(all(feature = "reqwest", feature = "axum"))]
#[tokio::test]
async fn flow() {
  use std::net::SocketAddr;
  use std::net::Ipv4Addr;

  let server = axum::Server::bind(&SocketAddr::from((Ipv4Addr::LOCALHOST, 0)))
    .serve(router().into_make_service());

  let client: ApiClient<SomeAPI> = ApiClient::new(
    reqwest::Url::parse(&format!("http://{}/", server.local_addr())).unwrap(),
    reqwest::Client::new()
  );

  // Prepare some signal for when the server should start shutting down...
  let (tx, rx) = tokio::sync::oneshot::channel::<()>();
  let graceful = server
    .with_graceful_shutdown(async {
        rx.await.ok();
    });

  tokio::spawn(async move {
    graceful.await.unwrap();
  });
  use tokio::time::{sleep, Duration};
  sleep(Duration::from_millis(100)).await;

  client.call_api(PostA(true)).await.unwrap().unwrap();
  let new_a = client.call_api(GetA).await.unwrap().unwrap();
  assert_eq!(new_a, true);

  let _ = tx.send(());
}

#[cfg(feature = "openapi-yaml")]
#[test]
fn openapi() {
  use serde_yaml::Value;

  let spec = serde_yaml::to_value(&gen_schema::<SomeAPI>()).unwrap();
  let spec_ref: Value = serde_yaml::from_str(r#"
    openapi: 3.1.0
    info:
      title: Some api
      summary: Some example api
      version: '0'
    paths:
      get_a:
        post:
          summary: Get A
          requestBody:
            content:
              application/json:
                schema:
                  type: 'null'
            required: true
          responses:
            default:
              description: Successfull response
              content:
                application/json:
                  schema:
                    oneOf:
                    - type: object
                      required:
                      - Ok
                      properties:
                        Ok:
                          type: boolean
                    - type: object
                      required:
                      - Err
                      properties:
                        Err:
                          type: integer
                          format: int32
      post_a:
        post:
          summary: Post A
          requestBody:
            content:
              application/json:
                schema:
                  type: boolean
            required: true
          responses:
            default:
              description: Successfull response
              content:
                application/json:
                  schema:
                    oneOf:
                    - type: object
                      required:
                      - Ok
                      properties:
                        Ok:
                          type: 'null'
                    - type: object
                      required:
                      - Err
                      properties:
                        Err:
                          type: integer
                          format: int32
    components: {}
    "#).unwrap();
  assert_eq!(spec, spec_ref);
}
