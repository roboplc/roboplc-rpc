use core::{fmt, marker::PhantomData};
use tracing::error;

use serde::{Deserialize, Serialize};

use crate::{
    dataformat::DataFormat,
    request::Request,
    response::{HandlerResponse, Response},
    RpcError, RpcResult,
};

const ERR_FAILED_TO_PARSE: &str = "Failed to parse RPC request";

/// JSON RPC server
#[allow(clippy::module_name_repetitions)]
pub struct RpcServer<'a, RPC: RpcServerHandler<'a>, M, SRC, R> {
    _phantom_a: PhantomData<&'a ()>,
    _phantom_m: PhantomData<M>,
    _phantom_src: PhantomData<SRC>,
    _phantom_r: PhantomData<R>,
    rpc: RPC,
}

impl<'a, RPC: RpcServerHandler<'a, Method = M, Result = R, Source = SRC>, M, SRC, R>
    RpcServer<'a, RPC, M, SRC, R>
where
    M: Deserialize<'a> + 'a,
    R: Serialize + Deserialize<'a> + 'a,
    SRC: fmt::Display,
{
    /// Create a new JSON RPC server
    pub fn new(rpc: RPC) -> Self {
        Self {
            _phantom_a: PhantomData,
            _phantom_m: PhantomData,
            _phantom_src: PhantomData,
            _phantom_r: PhantomData,
            rpc,
        }
    }
    /// Handle a JSON RPC request
    pub fn handle_request(&'a self, request: Request<M>, source: SRC) -> Option<Response<R>> {
        let result = match self.rpc.handle_call(request.method, source) {
            Ok(v) => HandlerResponse::Ok(v),
            Err(e) => HandlerResponse::Err(RpcError {
                kind: e.kind,
                message: e.message,
            }),
        };
        request
            .id
            .map(move |id| Response::from_handler_response(id, result))
    }
    /// Handle a JSON RPC request from a payload
    pub fn handle_request_payload<D>(&'a self, payload: &'a [u8], source: SRC) -> Option<Vec<u8>>
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
                                &Response::<R>::from_server_error(
                                    $response.id().clone(), error.to_string())) {
                            Some(response)
                        } else {
                            None
                        }
                    }
                }
            }};
        }
        match D::unpack::<Request<M>>(payload) {
            Ok(req) => self
                .handle_request(req, source)
                .and_then(|response| serialize_response!(response)),
            Err(error) => {
                error!(%source, %error, ERR_FAILED_TO_PARSE);
                if let Ok(invalid) = D::unpack::<crate::request::InvalidRequest>(payload) {
                    invalid
                        .into_response(error.to_string())
                        .and_then(|response: Response<R>| serialize_response!(response))
                } else {
                    None
                }
            }
        }
    }
}

/// RPC server trait
#[allow(clippy::module_name_repetitions)]
pub trait RpcServerHandler<'a> {
    /// Methods to handle
    type Method: Deserialize<'a>;
    /// Result of the methods
    type Result: Serialize + Deserialize<'a>;
    /// Source of the call (IP address, etc.)
    type Source: fmt::Display;

    /// A method to handle calls
    fn handle_call(&'a self, method: Self::Method, source: Self::Source)
        -> RpcResult<Self::Result>;
}
