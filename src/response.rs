use serde::{Deserialize, Serialize};

use crate::{
    de_validate_version, serialize_version, Id, RpcError, RpcErrorKind, RpcResult, String,
    VERSION_HEADER,
};

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
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
    #[serde(flatten)]
    rpc_response: RpcResponse<R>,
}

impl<R> Response<R> {
    pub fn into_parts(self) -> (Id, RpcResponse<R>) {
        (self.id, self.rpc_response)
    }
    pub fn from_rpc_response(id: Id, rpc_response: RpcResponse<R>) -> Response<R> {
        Response {
            jsonrpc: VERSION_HEADER,
            id,
            rpc_response,
        }
    }
    pub fn into_error_response(self, rpc_error: RpcError) -> Response<R> {
        Response {
            jsonrpc: VERSION_HEADER,
            id: self.id,
            rpc_response: RpcResponse::Error(rpc_error),
        }
    }
    pub fn id(&self) -> &Id {
        &self.id
    }
    pub fn into_server_error_response(self, error: String) -> Response<R> {
        Self::from_server_error(self.id, error)
    }
    pub fn from_server_error(id: Id, error: String) -> Response<R> {
        Response {
            jsonrpc: VERSION_HEADER,
            id,
            rpc_response: RpcResponse::Error(RpcError {
                kind: RpcErrorKind::InternalError,
                message: Some(error),
            }),
        }
    }
}

#[allow(clippy::module_name_repetitions)]
#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub enum RpcResponse<R> {
    #[cfg_attr(feature = "canonical", serde(rename = "error", alias = "e"))]
    #[cfg_attr(not(feature = "canonical"), serde(rename = "e"))]
    Error(RpcError),
    #[cfg_attr(feature = "canonical", serde(rename = "result", alias = "r"))]
    #[cfg_attr(not(feature = "canonical"), serde(rename = "r"))]
    Result(R),
}

impl<R> From<RpcResponse<R>> for RpcResult<R> {
    fn from(res: RpcResponse<R>) -> Self {
        match res {
            RpcResponse::Error(e) => Err(RpcError {
                kind: e.kind,
                message: e.message,
            }),
            RpcResponse::Result(r) => Ok(r),
        }
    }
}

impl<R> From<RpcResult<R>> for RpcResponse<R> {
    fn from(res: RpcResult<R>) -> Self {
        match res {
            Ok(r) => RpcResponse::Result(r),
            Err(e) => RpcResponse::Error(e),
        }
    }
}
