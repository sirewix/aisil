#[cfg(feature = "json-rpc-openrpc")]
pub use crate::json_rpc::openrpc;
#[cfg(feature = "post-json-openapi")]
pub use crate::post_json::openapi;
#[cfg(feature = "ts")]
pub mod ts;

/// Split docs into headers and the rest.
#[allow(dead_code)]
pub(crate) fn split_docs(docs: Option<&str>) -> (Option<String>, Option<String>) {
  docs
    .map(|docs| {
      docs
        .split_once("\n\n")
        .map_or((Some(docs.to_owned()), None), |(h, d)| (Some(h.to_owned()), Some(d.to_owned())))
    })
    .unwrap_or((None, None))
}
