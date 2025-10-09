# `aisil`

Typeful rust framework for defining simple APIs

This framework supports only narrow subset of HTTP spec, each method must be a POST request with a JSON body, returning a JSON. This constraint allows for abstracting over HTTP methods as over functions, that have one input and one output types. Such abstraction makes reasoning about API type safety much easier.

Note: every feature is optional, see [`Cargo.toml`](./Cargo.toml) for features reference

<!-- TOC GFM -->

* [Define API](#define-api)
* [Implement handler](#implement-handler)
* [Generate spec](#generate-spec)
* [Make client calls](#make-client-calls)
* [Derive TS types](#derive-ts-types)
* [Things to improve](#things-to-improve)

<!-- TOC -->

## Define API

```rust
/// Get A
#[derive(Clone, Serialize, Deserialize, JsonSchema, TS)]
pub struct GetA;

// not documented
#[derive(Clone, Serialize, Deserialize, JsonSchema, TS)]
pub struct PostA(pub bool);

/// Some example api
#[derive(DocumentedOpt)]
pub struct SomeAPI;

define_api! { SomeAPI => {
  /// Get A
  get_a, GetA => bool;
  // not documented method
  post_a, PostA => Res<()>;
} }

#[derive(DocumentedOpt)]
pub struct SomeAPI2;
```

## Implement handler

Implementing http service using `axum` with type checks:

```rust
#[derive(Clone)]
struct SomeBackend {
  a: Arc<Mutex<bool>>,
}

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

mk_handler! {SomeAPI, SomeBackend => {
  get_a : GetA,
  post_a : PostA,
}}

pub fn router() -> axum::Router {
  let env = SomeBackend {
    a: Arc::new(Mutex::new(false)),
  };
  crate::axum::mk_axum_router::<SomeAPI, SomeBackend>().with_state(env)
}
```

## Generate spec

Generating openapi spec for that API:

```rust
println!("{}", gen_yaml_openapi::<SomeAPI>());
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

## Things to improve

- [ ] Remove dependency on rust's `Result` as it's JSON representation is not really convinient for parsing in JS
- [ ] Allow for non-inlined TS types generation
- [ ] Debug `ts` feature
