use core::fmt;
use tracing::error;

use serde::{Deserialize, Serialize};

use crate::{
    dataformat::DataFormat,
    request::Request,
    response::{Response, RpcResponse},
    RpcError, RpcResult,
};

const ERR_FAILED_TO_PARSE: &str = "Failed to parse RPC request";

#[allow(clippy::module_name_repetitions)]
pub trait RpcServer<'a> {
    type Method: Deserialize<'a>;
    type Result: Serialize + Deserialize<'a>;
    type Source: fmt::Display;

    fn handle_request(
        &'a self,
        request: Request<Self::Method>,
        source: Self::Source,
    ) -> Option<Response<Self::Result>> {
        let result = match self.rpc_handler(request.method, source) {
            Ok(v) => RpcResponse::Ok(v),
            Err(e) => RpcResponse::Err(RpcError {
                kind: e.kind,
                message: e.message,
            }),
        };
        request
            .id
            .map(move |id| Response::from_rpc_response(id, result))
    }

    fn handle_request_payload<D>(
        &'a self,
        payload: &'a [u8],
        source: Self::Source,
    ) -> Option<Vec<u8>>
    where
        D: DataFormat,
    {
        macro_rules! serialize_response {
            ($response:expr) => {{
                match D::pack(&$response) {
                    Ok(v) => Some(v),
                    Err(error) => {
                        error!(%error, "Failed to serialize response");
                        if let Ok(response) = D::pack(
                                &Response::<Self::Result>::from_server_error(
                                    $response.id().clone(), error.to_string())) {
                            Some(response)
                        } else {
                            None
                        }
                    }
                }
            }};
        }
        match D::unpack::<Request<Self::Method>>(payload) {
            Ok(req) => self
                .handle_request(req, source)
                .and_then(|response| serialize_response!(response)),
            Err(error) => {
                error!(%source, %error, ERR_FAILED_TO_PARSE);
                if let Ok(invalid) = D::unpack::<crate::request::InvalidRequest>(payload) {
                    invalid
                        .into_response(error.to_string())
                        .and_then(|response: Response<Self::Result>| serialize_response!(response))
                } else {
                    None
                }
            }
        }
    }

    fn rpc_handler(&'a self, method: Self::Method, source: Self::Source)
        -> RpcResult<Self::Result>;
}
