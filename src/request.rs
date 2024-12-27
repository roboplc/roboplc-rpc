use serde::{Deserialize, Serialize};

use crate::{
    de_validate_version,
    response::{HandlerResponse, Response},
    serialize_version, Id, RpcError, RpcErrorKind, String, VERSION_HEADER,
};

#[cfg(feature = "canonical")]
use crate::{ERR_INVALID_PROTOCOL_VERSION, JSONRPC_VERSION};

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
/// JSON-RPC Request object
pub struct Request<M> {
    #[serde(
        default,
        deserialize_with = "de_validate_version",
        serialize_with = "serialize_version",
        skip_serializing_if = "Option::is_none"
    )]
    jsonrpc: Option<()>,
    #[cfg_attr(
        feature = "canonical",
        serde(skip_serializing_if = "Option::is_none", alias = "i")
    )]
    #[cfg_attr(
        not(feature = "canonical"),
        serde(rename = "i", skip_serializing_if = "Option::is_none")
    )]
    pub(crate) id: Option<Id>,
    #[cfg_attr(feature = "std", serde(flatten))]
    #[cfg_attr(not(feature = "std"), serde(rename = "p"))]
    pub(crate) method: M,
}

impl<'a, M> Request<M>
where
    M: Serialize + Deserialize<'a>,
{
    /// Create a new Request object with the given method with no ID (no response expected)
    pub fn new0(method: M) -> Request<M> {
        Request {
            jsonrpc: VERSION_HEADER,
            id: None,
            method,
        }
    }
    /// Create a new Request object with the given method and ID
    pub fn new(id: Id, method: M) -> Request<M> {
        Request {
            jsonrpc: VERSION_HEADER,
            id: Some(id),
            method,
        }
    }
    /// Split the Request object into its parts (useful for 3rd party serialization)
    pub fn into_parts(self) -> (Option<Id>, M) {
        (self.id, self.method)
    }
    /// Combine the parts into a Request object (useful for 3rd party de-serialization)
    pub fn from_parts(id: Option<Id>, method: M) -> Request<M> {
        Request {
            jsonrpc: VERSION_HEADER,
            id,
            method,
        }
    }
}

#[allow(clippy::module_name_repetitions)]
#[derive(Deserialize, Debug)]
/// An object to try de-serializing an invalid request to determine the error
pub struct InvalidRequest<'a> {
    #[allow(dead_code)]
    jsonrpc: Option<&'a str>,
    id: Option<Id>,
}

impl InvalidRequest<'_> {
    /// Convert the InvalidRequest object into a Response object with the given error message
    pub fn into_response<R>(self, error: String) -> Option<Response<R>> {
        if let Some(id) = self.id {
            #[cfg(feature = "canonical")]
            let (code, message) = if let Some(jsonrpc) = self.jsonrpc {
                if jsonrpc == JSONRPC_VERSION {
                    (RpcErrorKind::MethodNotFound, Some(error))
                } else {
                    (
                        RpcErrorKind::InvalidRequest,
                        #[allow(clippy::unnecessary_fallible_conversions)]
                        ERR_INVALID_PROTOCOL_VERSION.try_into().ok(),
                    )
                }
            } else {
                (RpcErrorKind::InvalidRequest, None)
            };
            #[cfg(not(feature = "canonical"))]
            let (code, message) = (RpcErrorKind::MethodNotFound, Some(error));
            Some(Response::from_handler_response(
                id,
                HandlerResponse::Err(RpcError {
                    kind: code,
                    message,
                }),
            ))
        } else {
            None
        }
    }
}
