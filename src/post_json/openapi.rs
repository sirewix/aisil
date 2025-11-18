//! OpenAPI spec generator for an API.
//!
//! Generates method definitions as `POST /<method_name>`
//!
//! Use [`gen_openapi`] or [`gen_openapi_yaml`].

use aide::openapi::*;
use documented::DocumentedOpt;
use indexmap::IndexMap;
use schemars::{JsonSchema, SchemaGenerator, generate::SchemaSettings};

use crate::generate::split_docs;
use crate::{HasMethod, IsApi};

/// API methods traversal trait for collecting methods and inserting request and
/// response schemas and their dependencies schema in [`SchemaGenerator`].
///
/// Don't use this trait directly, use [`gen_openapi`] or [`gen_openapi_yaml`]
/// instead.
pub trait GenerateOpenApi<API> {
  fn generate_openapi(
    paths: &mut IndexMap<String, ReferenceOr<PathItem>>,
    generator: &mut SchemaGenerator,
  );
}

impl<T, Res, API, N> GenerateOpenApi<API> for (T, N)
where
  T: JsonSchema,
  Res: JsonSchema,
  API: HasMethod<T, Res = Res>,
  N: GenerateOpenApi<API>,
{
  fn generate_openapi(
    paths: &mut IndexMap<String, ReferenceOr<PathItem>>,
    generator: &mut SchemaGenerator,
  ) {
    let req_schema = T::json_schema(generator);
    let res_schema = Res::json_schema(generator);

    let (summary, description) = split_docs(<API as HasMethod<T>>::METHOD_DOCS);
    paths.insert(
      API::METHOD_NAME.to_string(),
      ReferenceOr::Item(PathItem {
        post: Some(Operation {
          summary,
          description,
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

    N::generate_openapi(paths, generator);
  }
}

impl<API> GenerateOpenApi<API> for () {
  fn generate_openapi(
    _paths: &mut IndexMap<String, ReferenceOr<PathItem>>,
    _gen: &mut SchemaGenerator,
  ) {
  }
}

/// Generate OpenAPI schema.
///
/// ```ignore
/// gen_openapi<SomeAPI>
/// ```
pub fn gen_openapi<API>() -> OpenApi
where
  API::Methods: GenerateOpenApi<API>,
  API: IsApi + DocumentedOpt,
{
  let mut generator = SchemaGenerator::new(SchemaSettings::openapi3().with(|s| {
    s.definitions_path = "#/components/schemas/".into();
  }));
  let mut paths = IndexMap::new();
  API::Methods::generate_openapi(&mut paths, &mut generator);
  let (summary, description) = split_docs(API::DOCS);
  OpenApi {
    info: Info {
      title: API::API_NAME.into(),
      version: API::API_VERSION.into(),
      summary,
      description,
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
///
/// ```ignore
/// gen_openapi_yaml<SomeAPI>
/// ```
#[cfg(feature = "post-json-openapi-yaml")]
pub fn gen_openapi_yaml<API>() -> String
where
  API::Methods: GenerateOpenApi<API>,
  API: IsApi + DocumentedOpt,
{
  serde_yaml::to_string(&gen_openapi::<API>()).unwrap()
}

#[cfg(feature = "post-json-openapi-yaml")]
#[test]
fn test_openapi() {
  use crate::test::SomeAPI;
  use serde_yaml::Value;

  let spec = serde_yaml::to_value(gen_openapi::<SomeAPI>()).unwrap();
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
