use serde::{Deserialize, Serialize};

use crate::{
    de_validate_version,
    response::{Response, RpcResponse},
    serialize_version, Id, RpcError, RpcErrorKind, String, ERR_INVALID_PROTOCOL_VERSION,
    JSONRPC_VERSION, VERSION_HEADER,
};

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
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
    #[serde(flatten)]
    pub(crate) method: M,
}

impl<'a, M> Request<M>
where
    M: Serialize + Deserialize<'a>,
{
    pub fn new0(method: M) -> Request<M> {
        Request {
            jsonrpc: VERSION_HEADER,
            id: None,
            method,
        }
    }
    pub fn new(id: Id, method: M) -> Request<M> {
        Request {
            jsonrpc: VERSION_HEADER,
            id: Some(id),
            method,
        }
    }
    pub fn into_parts(self) -> (Option<Id>, M) {
        (self.id, self.method)
    }
}

#[allow(clippy::module_name_repetitions)]
#[derive(Deserialize, Debug)]
pub struct InvalidRequest<'a> {
    jsonrpc: Option<&'a str>,
    id: Option<Id>,
}

impl InvalidRequest<'_> {
    pub fn into_response<R>(self, error: String) -> Option<Response<R>> {
        if let Some(id) = self.id {
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
            Some(Response::from_rpc_response(
                id,
                RpcResponse::Error(RpcError {
                    kind: code,
                    message,
                }),
            ))
        } else {
            None
        }
    }
}
