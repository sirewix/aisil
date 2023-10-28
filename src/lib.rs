#[cfg(feature = "reqwest")]
use std::marker::PhantomData;
#[cfg(feature = "openapi")]
use aide::openapi::*;
#[cfg(feature = "openapi")]
use schemars::{
  gen::{SchemaGenerator, SchemaSettings},
  JsonSchema,
};
#[cfg(feature = "openapi")]
use indexmap::IndexMap;
#[cfg(test)]
mod test;

#[macro_export]
macro_rules! def_method {
  {$api:ty, $req:ty, $res:ty, $desc:expr, $method:expr} => {
      impl $crate::ApiMethod<$api> for $req {
        type Res = $res;
        const NAME: &'static str = $method;
        const DESCRIPTION: &'static str = $desc;
      }
  }
}

#[macro_export]
macro_rules! build_hlist {
  () => { $crate::Nil };
  ($type:ty $(, $rest:ty)*) => { $crate::Cons<$type, $crate::build_hlist!($($rest),*)> };
}

#[macro_export]
macro_rules! define_api {
  {  $api:ty, $name:expr, $api_desc:expr => $err:ty { $($method:expr, $req:ty => $res:ty : $desc:expr;)+ }} => (
      $( $crate::def_method!{$api, $req, Result<$res, $err>, $desc, $method} )+

      impl $crate::IsApi for $api {
        type MethodList = $crate::build_hlist!($($req),+);
        const NAME: &'static str = $name;
        const DESCRIPTION: &'static str = $api_desc;
      }
  );
}

#[macro_export]
macro_rules! impl_method {
  ($api:ty : $func:ident : $p:pat = $req:ty => $body: expr) => (
    pub async fn $func($p: $req) -> <$req as $crate::ApiMethod<$api>>::Res {
      $body
    }
  )
}

#[macro_export]
macro_rules! impl_env_method {
  ($api:ty : $func:ident : $p:pat = $req:ty => $body: expr) => (
    pub async fn $func($p: $req) -> <$req as $crate::ApiMethod<$api>>::Res {
      $body
    }
  )
}

#[cfg(feature = "axum")]
#[macro_export]
macro_rules! mk_axum_router {
  ($api:ty, $env:expr, $envt:ty => { $($func:ident : $req:ty ,)+ } ) => (
    axum::Router::new()
      $( .route(
            &format!("/{}", <$req as $crate::ApiMethod<$api>>::NAME),
            axum::routing::post(|
              axum::extract::State(env): axum::extract::State<$envt>,
              axum::extract::Json(request): axum::extract::Json<$req>
            | async move {
                axum::extract::Json(
                  (env.$func(request).await.map_err(Into::into))
                   as <$req as $crate::ApiMethod<$api>>::Res
              )})
      ) )+
      .with_state($env.to_owned())
  )
}

#[cfg(feature = "reqwest")]
pub struct ApiClient<API> {
  base_url: reqwest::Url,
  client: reqwest::Client,
  api_marker: PhantomData<API>
}

#[cfg(feature = "reqwest")]
impl<API> Clone for ApiClient<API> {
    fn clone(&self) -> Self {
      ApiClient {
        base_url: self.base_url.clone(),
        client: self.client.clone(),
        api_marker: PhantomData,
      }
    }
}

#[cfg(feature = "reqwest")]
impl<API> ApiClient<API> {
  pub fn new(base_url: reqwest::Url, client: reqwest::Client) -> Self {
    ApiClient {
      base_url,
      client,
      api_marker: PhantomData,
    }
  }

  pub async fn call_api<
    Req: ApiMethod<API, Res = Res> + serde::Serialize,
    Res: for<'a> serde::Deserialize<'a>,
  >(
    &self,
    req: Req,
  ) -> Result<Res, reqwest::Error> {
    Ok(
      self
        .client
        .post(self.base_url.join(<Req as ApiMethod<API>>::NAME).unwrap())
        .json(&req)
        .send()
        .await?
        .json::<<Req as ApiMethod<API>>::Res>()
        .await?,
    )
  }
}

pub trait IsApi {
  type MethodList;
  const DESCRIPTION: &'static str;
  const NAME: &'static str;
}

pub trait ApiMethod<I> {
  type Res;
  const DESCRIPTION: &'static str;
  const NAME: &'static str;
}

pub struct Cons<T, N> (T, N);
pub struct Nil {}

#[cfg(feature = "openapi")]
pub trait InsertPathItems<API> {
  fn insert_path_item(
    paths: &mut IndexMap<String, ReferenceOr<PathItem>>,
    gen: &mut SchemaGenerator,
  );
}

#[cfg(feature = "openapi")]
impl <
  T: ApiMethod<API, Res = Res> + JsonSchema,
  Res: JsonSchema,
  API,
  N: InsertPathItems<API>
  > InsertPathItems<API> for Cons<T, N> {
  fn insert_path_item(
    paths: &mut IndexMap<String, ReferenceOr<PathItem>>,
    gen: &mut SchemaGenerator,
  ) {
    let req_schema = T::json_schema(gen);
    let res_schema = Res::json_schema(gen);

    paths.insert(<T as ApiMethod::<API>>::NAME.to_string(), ReferenceOr::Item(
        PathItem {
          post: Some(Operation {
            summary: Some(<T as ApiMethod::<API>>::DESCRIPTION.to_string()),
            request_body: Some(ReferenceOr::Item(
              RequestBody {
                required: true,
                content: IndexMap::from_iter([("application/json".into(),
                  MediaType {
                    schema: Some(SchemaObject {
                        json_schema: req_schema,
                        example: None,
                        external_docs: None,
                    }), ..Default::default()
                  },
                )]), ..RequestBody::default()
              }
            )),
            responses: Some (Responses {
              default: Some(ReferenceOr::Item(
                Response {
                  description: "Successfull response".into(),
                  content: IndexMap::from_iter([("application/json".into(),
                    MediaType {
                      schema: Some(SchemaObject {
                          json_schema: res_schema,
                          example: None,
                          external_docs: None,
                      }), ..Default::default()
                    },
                  )]), ..Default::default()
                }
              )), ..Default::default()
            }), ..Operation::default()
          }), ..PathItem::default()
        }
    ));

    N::insert_path_item(paths, gen);
  }
}

#[cfg(feature = "openapi")]
impl<API> InsertPathItems<API> for Nil {
  fn insert_path_item(
    _paths: &mut IndexMap<String, ReferenceOr<PathItem>>,
    _gen: &mut SchemaGenerator,
  ) { }
}

#[cfg(feature = "openapi")]
pub fn gen_schema<API>() -> OpenApi
where
  API::MethodList: InsertPathItems<API>,
  API: IsApi,
{
  let mut gen = SchemaGenerator::new(SchemaSettings::draft07().with(|s| {
    s.definitions_path = "#/components/schemas/".into();
  }));
  let mut paths = IndexMap::new();
  API::MethodList::insert_path_item(&mut paths, &mut gen);
  OpenApi {
    info: Info {
      title: API::NAME.into(),
      version: "0".into(),
      summary: Some(API::DESCRIPTION.into()),
      ..Default::default()
    },
    paths: Some(Paths {
      paths: paths,
      ..Default::default()
    }),
    components: Some(Components {
      schemas: gen
        .take_definitions()
        .into_iter()
        .map(|(k, v)| {
          (
            k,
            SchemaObject {
              json_schema: v,
              example: None,
              external_docs: None,
            },
          )
        })
        .collect(),
      ..Default::default()
    }),
    ..Default::default()
  }
}

#[cfg(feature = "openapi-yaml")]
pub fn gen_yaml_openapi<API>() -> String
where
  API::MethodList: InsertPathItems<API>,
  API: IsApi,
{
  serde_yaml::to_string(&gen_schema::<API>()).unwrap()
}

#[cfg(feature = "ts")]
use ts_rs::TS;

#[cfg(feature = "ts")]
pub struct TsMethod {
  pub method_name: String,
  pub request_type: String,
  pub response_type: String,
}

#[cfg(feature = "ts")]
pub struct TsApi {
  pub types: Vec<String>,
  pub methods: Vec<TsMethod>,
}

#[cfg(feature = "ts")]
pub trait TraverseTsClient<API> {
  fn add_methods(ts_api: &mut TsApi);
}

#[cfg(feature = "ts")]
impl<API> TraverseTsClient<API> for Nil {
  fn add_methods(_ts_api: &mut TsApi) {}
}

#[cfg(feature = "ts")]
impl <
  T: ApiMethod<API, Res = Result<Res, Err>> + TS,
  Err: TS,
  Res: TS,
  API,
  N: TraverseTsClient<API>
  > TraverseTsClient<API> for Cons<T, N> {
  fn add_methods(ts_api: &mut TsApi) {
    ts_api.methods.push(TsMethod {
      method_name: <T as ApiMethod<API>>::NAME.into(),
      request_type: <T as TS>::inline(),
      response_type: format!("Result<{}, {}>", <Res as TS>::inline(), <Err as TS>::inline()),
    });
    N::add_methods(ts_api);
  }
}

#[cfg(feature = "ts")]
pub fn gen_ts_api<API>() -> String
where
  API::MethodList: TraverseTsClient<API>,
  API: IsApi,
{
  let mut ts_api = TsApi {
    types: Vec::new(),
    methods: Vec::new(),
  };
  ts_api.types.push("type Result<R, E> = {Ok: R} | {Err: E};".into());
  API::MethodList::add_methods(&mut ts_api);
  vec![
    ts_api.types.join("\n"),
    "type Request<M> = ".into(),
    ts_api.methods.iter().map(|method| {
      format!(
        "  '{}' extends M ? {} :",
        method.method_name,
        method.request_type,
      )
    }).collect::<Vec<_>>().join("\n"),
    "  void;\n".into(),

    "type Response<M> = ".into(),
    ts_api.methods.iter().map(|method| {
      format!(
        "  '{}' extends M ? {} :",
        method.method_name,
        method.response_type,
      )
    }).collect::<Vec<_>>().join("\n"),
    "  void;".into(),
  ].join("\n")
}

