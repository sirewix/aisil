//! OpenAPI spec generator for an API.
//!
//! Generates method definitions as `POST /<method_name>`
//!
//! Use [`gen_openapi`] or [`gen_openapi_yaml`].

use crate::{Cons, HasDocumentedMethod, HasMethod, IsApi, Nil};
use aide::openapi::*;
use documented::DocumentedOpt;
use indexmap::IndexMap;
use schemars::{JsonSchema, SchemaGenerator, generate::SchemaSettings};

/// API methods traversal trait for collecting methods and inserting request and
/// response schemas and their dependencies schema in [`SchemaGenerator`].
pub trait InsertPathItems<API, MS> {
  fn insert_path_item(
    paths: &mut IndexMap<String, ReferenceOr<PathItem>>,
    generator: &mut SchemaGenerator,
  );
}

impl<
  T: JsonSchema,
  Res: JsonSchema,
  API: HasMethod<T, Res = Res> + HasDocumentedMethod<T, TS>,
  N: InsertPathItems<API, NS>,
  TS,
  NS,
> InsertPathItems<API, (TS, NS)> for Cons<T, N>
{
  fn insert_path_item(
    paths: &mut IndexMap<String, ReferenceOr<PathItem>>,
    generator: &mut SchemaGenerator,
  ) {
    let req_schema = T::json_schema(generator);
    let res_schema = Res::json_schema(generator);

    paths.insert(
      API::METHOD_NAME.to_string(),
      ReferenceOr::Item(PathItem {
        post: Some(Operation {
          summary: <API as HasDocumentedMethod<T, TS>>::DOCS.map(|d| d.to_string()),
          request_body: Some(ReferenceOr::Item(RequestBody {
            required: true,
            content: IndexMap::from_iter([(
              "application/json".into(),
              MediaType {
                schema: Some(SchemaObject {
                  json_schema: req_schema,
                  example: None,
                  external_docs: None,
                }),
                ..Default::default()
              },
            )]),
            ..RequestBody::default()
          })),
          responses: Some(Responses {
            default: Some(ReferenceOr::Item(Response {
              description: "Successful response".into(),
              content: IndexMap::from_iter([(
                "application/json".into(),
                MediaType {
                  schema: Some(SchemaObject {
                    json_schema: res_schema,
                    example: None,
                    external_docs: None,
                  }),
                  ..Default::default()
                },
              )]),
              ..Default::default()
            })),
            ..Default::default()
          }),
          ..Operation::default()
        }),
        ..PathItem::default()
      }),
    );

    N::insert_path_item(paths, generator);
  }
}

#[cfg(feature = "openapi")]
impl<API> InsertPathItems<API, ()> for Nil {
  fn insert_path_item(
    _paths: &mut IndexMap<String, ReferenceOr<PathItem>>,
    _gen: &mut SchemaGenerator,
  ) {
  }
}

/// Generate OpenAPI schema.
pub fn gen_openapi<API, MS>() -> OpenApi
where
  API::MethodList: InsertPathItems<API, MS>,
  API: IsApi + DocumentedOpt,
{
  let mut generator = SchemaGenerator::new(SchemaSettings::openapi3().with(|s| {
    s.definitions_path = "#/components/schemas/".into();
  }));
  let mut paths = IndexMap::new();
  API::MethodList::insert_path_item(&mut paths, &mut generator);
  OpenApi {
    info: Info {
      title: API::API_NAME.into(),
      version: API::API_VERSION.into(),
      summary: API::DOCS.map(|d| d.into()),
      ..Default::default()
    },
    paths: Some(Paths { paths, ..Default::default() }),
    components: Some(Components {
      schemas: generator
        .take_definitions(true)
        .into_iter()
        .map(|(k, v)| {
          let json_schema = v.try_into().unwrap();
          (k, SchemaObject { json_schema, example: None, external_docs: None })
        })
        .collect(),
      ..Default::default()
    }),
    ..Default::default()
  }
}

/// [`gen_openapi`] wrapper that produces OpenAPI spec as YAML string.
#[cfg(feature = "openapi-yaml")]
pub fn gen_openapi_yaml<API, MS>() -> String
where
  API::MethodList: InsertPathItems<API, MS>,
  API: IsApi + DocumentedOpt,
{
  serde_yaml::to_string(&gen_openapi::<API, MS>()).unwrap()
}

#[cfg(feature = "openapi-yaml")]
#[test]
fn test_openapi() {
  use super::openapi::gen_openapi;
  use crate::test::SomeAPI;
  use serde_yaml::Value;

  let spec = serde_yaml::to_value(gen_openapi::<SomeAPI, _>()).unwrap();
  let spec_ref: Value = serde_yaml::from_str(
    r#"
    openapi: 3.1.0
    info:
      title: SomeAPI
      summary: Some example api
      version: '0.0.0'
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
              description: Successful response
              content:
                application/json:
                  schema:
                    type: boolean
      post_a:
        post:
          # summary: Post A
          requestBody:
            content:
              application/json:
                schema:
                  type: boolean
            required: true
          responses:
            default:
              description: Successful response
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
                          type: string
    components: {}
    "#,
  )
  .unwrap();
  assert_eq!(spec, spec_ref);
}
