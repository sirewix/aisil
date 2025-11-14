//! OpenRPC spec generator for an API.
//!
//! Use [`gen_openrpc`] or [`gen_openrpc_yaml`].

use documented::DocumentedOpt;
use schemars::{JsonSchema, Schema, SchemaGenerator, generate::SchemaSettings};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

use crate::generate::split_docs;
use crate::{HasDocumentedMethod, HasMethod, IsApi};

/// Generate OpenRPC schema.
pub fn gen_openrpc<API, SP>() -> impl Serialize
where
  API: IsApi + DocumentedOpt,
  API::Methods: GenerateOpenRpc<API, SP>,
{
  let mut methods = Vec::new();
  let mut generator = SchemaSettings::draft07().into_generator();
  API::Methods::generate_openrpc(&mut methods, &mut generator);
  OpenRpc {
    openrpc: "1.3.0".into(), // this is sort of random
    info: Info {
      title: API::API_NAME.into(),
      version: API::API_VERSION.into(),
      description: <API as DocumentedOpt>::DOCS.map(ToOwned::to_owned),
    },
    methods,
    components: Some(Components {
      schemas: generator.take_definitions(true).into_iter().collect(),
    }),
  }
}

/// [`gen_openrpc`] wrapper that produces OpenRpc spec as YAML string.
///
/// ```ignore
/// gen_openrpc_yaml<SomeAPI, _>
/// ```
#[cfg(feature = "json-rpc-openrpc-yaml")]
pub fn gen_openrpc_yaml<API, MS>() -> String
where
  API::Methods: GenerateOpenRpc<API, MS>,
  API: IsApi + DocumentedOpt,
{
  serde_yaml::to_string(&gen_openrpc::<API, MS>()).unwrap()
}

/// Don't use this trait directly, use [`gen_openrpc`] or [`gen_openrpc_yaml`]
/// instead.
pub trait GenerateOpenRpc<API, SP> {
  fn generate_openrpc(methods: &mut Vec<OpenRpcMethodDoc>, generator: &mut SchemaGenerator);
}

impl<API, H, T, DM, DMT> GenerateOpenRpc<API, (DM, DMT)> for (H, T)
where
  API: IsApi + HasMethod<H> + HasDocumentedMethod<H, DM>,
  H: JsonSchema,
  <API as HasMethod<H>>::Res: JsonSchema,
  T: GenerateOpenRpc<API, DMT>,
{
  fn generate_openrpc(methods: &mut Vec<OpenRpcMethodDoc>, generator: &mut SchemaGenerator) {
    let (summary, description) = split_docs(<API as HasDocumentedMethod<H, DM>>::DOCS);
    let doc = OpenRpcMethodDoc {
      name: API::METHOD_NAME.into(),
      summary,
      description,
      params: vec![ContentDescriptor {
        name: "payload".into(),
        summary: None,
        description: None,
        required: true,
        schema: <H as JsonSchema>::json_schema(generator),
        // deprecated: false,
      }],
      result: Some(ContentDescriptor {
        name: "result".into(),
        summary: None,
        description: None,
        required: true,
        schema: <<API as HasMethod<H>>::Res as JsonSchema>::json_schema(generator),
        // deprecated: false,
      }),
      // deprecated: false,
      param_structure: ParamStructure::ByName,
    };
    methods.push(doc);
    T::generate_openrpc(methods, generator);
  }
}

impl<API> GenerateOpenRpc<API, ()> for () {
  fn generate_openrpc(_: &mut Vec<OpenRpcMethodDoc>, _: &mut SchemaGenerator) {}
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct ContentDescriptor {
  name: String,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  summary: Option<String>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  description: Option<String>,
  required: bool,
  schema: Schema,
  // deprecated: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct OpenRpcMethodDoc {
  name: String,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  summary: Option<String>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  description: Option<String>,
  params: Vec<ContentDescriptor>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  result: Option<ContentDescriptor>,
  //deprecated: bool,
  param_structure: ParamStructure,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
#[serde(rename_all = "kebab-case")]
enum ParamStructure {
  ByName,
  ByPosition,
  #[default]
  Either,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct OpenRpc {
  openrpc: String,
  info: Info,
  methods: Vec<OpenRpcMethodDoc>,
  components: Option<Components>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
struct Components {
  schemas: BTreeMap<String, Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Info {
  title: String,
  version: String,
  description: Option<String>,
}

#[cfg(test)]
#[cfg(feature = "json-rpc-openrpc-yaml")]
mod tests {
  use crate::test::SomeAPI;
  use serde_yaml::Value;

  #[test]
  fn openrpc() {
    let spec = serde_yaml::to_value(super::gen_openrpc::<SomeAPI, _>()).unwrap();
    let spec_ref: Value = serde_yaml::from_str(
      r#"
      openrpc: 1.3.0
      info:
        title: SomeAPI
        version: 0.0.0
        description: Some example api
      methods:
      - name: get_a
        summary: Get A
        params:
        - name: payload
          required: true
          schema:
            type: 'null'
        result:
          name: result
          required: true
          schema:
            type: boolean
        paramStructure: by-name
      - name: post_a
        params:
        - name: payload
          required: true
          schema:
            type: boolean
        result:
          name: result
          required: true
          schema:
            oneOf:
            - type: object
              properties:
                Ok:
                  type: 'null'
              required:
              - Ok
            - type: object
              properties:
                Err:
                  type: string
              required:
              - Err
        paramStructure: by-name
      components:
        schemas: {}
      "#,
    )
    .unwrap();
    assert_eq!(spec, spec_ref);
  }
}
