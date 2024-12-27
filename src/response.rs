use serde::{Deserialize, Serialize};

use crate::{
    de_validate_version, serialize_version, Id, RpcError, RpcErrorKind, RpcResult, String,
    VERSION_HEADER,
};

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
/// JSON-RPC Response object
pub struct Response<R> {
    #[serde(
        default,
        deserialize_with = "de_validate_version",
        serialize_with = "serialize_version",
        skip_serializing_if = "Option::is_none"
    )]
    jsonrpc: Option<()>,
    #[cfg_attr(feature = "canonical", serde(alias = "i"))]
    #[cfg_attr(not(feature = "canonical"), serde(rename = "i"))]
    id: Id,
    #[cfg_attr(feature = "std", serde(flatten))]
    #[cfg_attr(not(feature = "std"), serde(rename = "p"))]
    handler_response: HandlerResponse<R>,
}

impl<R> Response<R> {
    /// Split the Response object into its parts (useful for 3rd party serialization)
    pub fn into_parts(self) -> (Id, HandlerResponse<R>) {
        (self.id, self.handler_response)
    }
    /// Combine the parts into a Response object (useful for 3rd party de-serialization)
    pub fn from_parts(id: Id, handler_response: HandlerResponse<R>) -> Response<R> {
        Response {
            jsonrpc: VERSION_HEADER,
            id,
            handler_response,
        }
    }
    /// Create a new Response object with the given ID and result from the RPC handler response
    pub fn from_handler_response(id: Id, handler_response: HandlerResponse<R>) -> Response<R> {
        Response {
            jsonrpc: VERSION_HEADER,
            id,
            handler_response,
        }
    }
    /// Convert the response into an error response with the given error
    pub fn into_error_response(self, rpc_error: RpcError) -> Response<R> {
        Response {
            jsonrpc: VERSION_HEADER,
            id: self.id,
            handler_response: HandlerResponse::Err(rpc_error),
        }
    }
    /// Get the ID of the response
    pub fn id(&self) -> &Id {
        &self.id
    }
    /// Get the handler response
    pub fn into_server_error_response(self, error: String) -> Response<R> {
        Self::from_server_error(self.id, error)
    }
    /// Create a new Response object with the given ID and error message
    pub fn from_server_error(id: Id, error: String) -> Response<R> {
        Response {
            jsonrpc: VERSION_HEADER,
            id,
            handler_response: HandlerResponse::Err(RpcError {
                kind: RpcErrorKind::InternalError,
                message: Some(error),
            }),
        }
    }
}

#[allow(clippy::module_name_repetitions)]
#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
/// RPC handler response object. Basically duplicates the standard Result object, required for the
/// proper serialization
pub enum HandlerResponse<R> {
    #[cfg_attr(feature = "canonical", serde(rename = "result", alias = "r"))]
    #[cfg_attr(not(feature = "canonical"), serde(rename = "r"))]
    /// The RPC handler returned a data
    Ok(R),
    #[cfg_attr(feature = "canonical", serde(rename = "error", alias = "e"))]
    #[cfg_attr(not(feature = "canonical"), serde(rename = "e"))]
    /// The RPC handler returned an error
    Err(RpcError),
}

impl<R> HandlerResponse<R> {
    /// Is the response Ok
    pub fn is_ok(&self) -> bool {
        matches!(self, HandlerResponse::Ok(_))
    }
    /// Is the response an error
    pub fn is_err(&self) -> bool {
        matches!(self, HandlerResponse::Err(_))
    }
    /// Convert the response data into an option
    pub fn ok(&self) -> Option<&R> {
        match self {
            HandlerResponse::Ok(r) => Some(r),
            HandlerResponse::Err(_) => None,
        }
    }
    /// Convert the response error into an option
    pub fn err(&self) -> Option<&RpcError> {
        match self {
            HandlerResponse::Ok(_) => None,
            HandlerResponse::Err(e) => Some(e),
        }
    }
}

impl<R> From<HandlerResponse<R>> for RpcResult<R> {
    fn from(res: HandlerResponse<R>) -> Self {
        match res {
            HandlerResponse::Err(e) => Err(RpcError {
                kind: e.kind,
                message: e.message,
            }),
            HandlerResponse::Ok(r) => Ok(r),
        }
    }
}

impl<R> From<RpcResult<R>> for HandlerResponse<R> {
    fn from(res: RpcResult<R>) -> Self {
        match res {
            Ok(r) => HandlerResponse::Ok(r),
            Err(e) => HandlerResponse::Err(e),
        }
    }
}
