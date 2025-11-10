# `aisil`

Lightweight framework to define APIs as types.

`aisil` is designed to be transport and protocol agnostic. At the moment,
however, only one transport protocol is supported (HTTP's `POST /<method_name>`
with json bodies). Feel free to extend the base framework with whatever fits
your requirements.


<!-- START doctoc generated TOC please keep comment here to allow auto update -->
<!-- DON'T EDIT THIS SECTION, INSTEAD RE-RUN doctoc TO UPDATE -->

- [Define API](#define-api)
- [Implement service](#implement-service)
- [Expose service](#expose-service)
- [Make client calls](#make-client-calls)
- [Generate spec](#generate-spec)
- [Derive TS types](#derive-ts-types)
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
  get_a, GetA => bool;

  /// Post A
  post_a, PostA => Result<(), String>;
} }
```

## Implement service

```rust
#[derive(Clone)]
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

```rust
pub fn router() -> axum::Router {
  let backend = SomeBackend { a: Arc::new(Mutex::new(false)), };
  crate::axum::mk_axum_router::<SomeAPI, SomeBackend>().with_state(backend)
}
```

## Make client calls

Use that API to make type safe client calls:

```rust
use reqwest::{Url, Client};
let client = ApiClient::new(Url::parse(client_url).unwrap(), Client::new());
client.call_api(PostA(true)).await.unwrap().unwrap();
let new_a = client.call_api(GetA).await.unwrap().unwrap();
assert_eq!(new_a, true);
```

## Generate spec

Generating openapi spec for that API:

```rust
println!("{}", gen_yaml_openapi::<SomeAPI>());
```

## Derive TS types

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

- [ ] json-rpc + openrpc
- [ ] Allow for non-inlined TS types generation
- [ ] Debug `ts` feature
- [ ] no-std feature
