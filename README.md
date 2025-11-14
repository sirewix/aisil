# `aisil`

Lightweight framework to define APIs as types.

`aisil` is designed to be transport and protocol agnostic. At the moment,
however, only one transport protocol is supported (HTTP's `POST /<method_name>`
with json bodies). Feel free to extend the base framework with whatever fits
your requirements.

See docs at [docs.rs/aisil](https://docs.rs/aisil/latest).

<!-- START doctoc generated TOC please keep comment here to allow auto update -->
<!-- DON'T EDIT THIS SECTION, INSTEAD RE-RUN doctoc TO UPDATE -->

- [Define API](#define-api)
- [Implement service](#implement-service)
- [Expose service](#expose-service)
- [Make client calls](#make-client-calls)
- [Generate spec](#generate-spec)
- [Generate TS types](#generate-ts-types)
- [Things to implement/improve](#things-to-implementimprove)

<!-- END doctoc generated TOC please keep comment here to allow auto update -->

## Define API

- A **method** is defined as `Request` → (method name, `Response`)
  dependency in the context of an API (See `HasMethod` trait).

- An **API** is defined as `ApiMetaType` → `[*]` dependency (See `IsApi` trait),
  where `[*]` is a heterogeneous list of request types that belong to the API.

An example of an API definition with two methods:

```rust
/// Get A
#[derive(Serialize, Deserialize, JsonSchema, TS, DocumentedOpt)]
pub struct GetA;

#[derive(Serialize, Deserialize, JsonSchema, TS)]
pub struct PostA(pub bool);

/// Some example api
#[derive(DocumentedOpt)]
pub struct SomeAPI;

define_api! { SomeAPI => {
  // <method_name>, <RequestType> => <ResponseType>;

  // documentation for this method will be taken from DocumentedOpt
  get_a, GetA => bool;

  /// Post A
  post_a, PostA => Result<(), String>;
} }
```

## Implement service

```rust
#[derive(Clone, Default)]
struct SomeBackend {
  a: Arc<Mutex<bool>>,
}

impl ImplsMethod<SomeAPI, GetA> for SomeBackend {
  async fn call_api(&self, _: GetA) -> bool {
    self.a.lock().await.clone()
  }
}

impl ImplsMethod<SomeAPI, PostA> for SomeBackend {
  async fn call_api(&self, PostA(new_a): PostA) -> Result<(), String> {
    let mut a = self.a.lock().await;
    (!*a).then_some(()).ok_or("can't post `a` anymore".to_owned())?;
    *a = new_a;
    Ok(())
  }
}
```

## Expose service

As HTTP `POST /<method_name>`:

```rust
pub fn router() -> axum::Router {
  let backend = SomeBackend::default();
  aisil::post_json::mk_post_json_router::<SomeAPI, SomeBackend>().with_state(backend)
}
```

or as JsonRPC:

```rust
let backend = SomeBackend::default();
Router::new().route(
  "/rpc",
  post(async move |State(svc), Json(request): Json<server::JsonRpcRequest>| {
    Json(aisil::server::json_rpc::json_rpc_router::<SomeAPI, SomeBackend>(&svc, request).await)
  }),
).with_state(state);
```

## Make client calls

Use that API to make type safe client calls:

Either HTTP `POST /<method_name>`:

```rust
let client = PostJsonClient::new(Url::parse(client_url)?, reqwest::Client::new());
client.call_api(PostA(true)).await?.unwrap();
let new_a = client.call_api(GetA).await?;
assert_eq!(new_a, true);
```

or as JsonRPC:

```rust
let client = JsonRpcClient::new(Method::POST, Url::parse(client_url)?, reqwest::Client::new());
client.call_api(PostA(true)).await?.unwrap();
let new_a = client.call_api(GetA).await?;
assert_eq!(new_a, true);
```

## Generate spec

OpenAPI for HTTP `POST /<method_name>`:


```rust
println!("{}", gen_openapi_yaml::<SomeAPI, _>());
```

OpenRPC for JsonRPC:

```rust
println!("{}", gen_openrpc_yaml::<SomeAPI, _>());
```

## Generate TS types

```rust
println!("{}", gen_ts_api::<SomeAPI>());
```

Current implementation works by inlining everything, which is probably undesirable:

```ts
type Request<M> =
  M extends 'get_a' ? null :
  M extends 'post_a' ? boolean :
  void;

type Response<M> =
  M extends 'get_a' ? Result<boolean, number> :
  M extends 'post_a' ? Result<null, number> :
  void;
```

TS boilerplate would look something like this:

```ts
const callSomeApi<M> = async (req: Request<M>) => {
  const raw_response = await fetch(`http://example.com/{method}`, {
    method: 'POST',
    body: req,
    headers: { 'Content-Type': 'application/json' }
  });
  const json = await raw_response.json();
  json as Response<M>
}
```

And to unwrap rust's `Result`:

```ts
function unwrapResult<R, E>(a: Result<R, E>): R {
  if ('Ok' in a) {
    return a.Ok;
  } else if ('Err' in a) {
    throw  Error(JSON.stringify(a.Err))
  } else {
    throw Error('non api error')
  }
}
```

## Things to implement/improve

- [ ] Allow for non-inlined TS types generation
- [ ] Debug `ts` feature
- [ ] no-std feature
