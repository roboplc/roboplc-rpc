use core::{
    mem,
    sync::atomic::{AtomicU32, Ordering},
};

use serde::{Deserialize, Serialize};

use crate::{dataformat, request::Request, response::Response, RpcError, RpcErrorKind, RpcResult};

#[allow(clippy::module_name_repetitions)]
#[derive(Default)]
pub struct RpcClient<'a, D, M, R>
where
    D: dataformat::DataFormat,
    M: Serialize + Deserialize<'a>,
    R: Serialize + Deserialize<'a>,
{
    _phantom_d: core::marker::PhantomData<D>,
    _phantom_a: core::marker::PhantomData<&'a ()>,
    _phantom_m: core::marker::PhantomData<M>,
    _phantom_r: core::marker::PhantomData<R>,
    request_id: AtomicU32,
}

impl<'a, D, M, R> RpcClient<'a, D, M, R>
where
    D: dataformat::DataFormat,
    M: Serialize + Deserialize<'a>,
    R: Serialize + Deserialize<'a>,
{
    pub fn new() -> Self {
        Self {
            _phantom_d: core::marker::PhantomData,
            _phantom_a: core::marker::PhantomData,
            _phantom_m: core::marker::PhantomData,
            _phantom_r: core::marker::PhantomData,
            request_id: AtomicU32::new(0),
        }
    }
    pub fn request(&self, method: M) -> Result<RpcClientRequest<D, M, R>, D::PackError> {
        let id = self.request_id.fetch_add(1, Ordering::SeqCst);
        let req = Request::new(id.into(), method);
        let payload = D::pack(&req)?;
        Ok(RpcClientRequest::new(Some(id), payload))
    }
    pub fn request0(&self, method: M) -> Result<RpcClientRequest<D, M, R>, D::PackError> {
        let req = Request::new0(method);
        let payload = D::pack(&req)?;
        Ok(RpcClientRequest::new(None, payload))
    }
}

pub struct RpcClientRequest<D, M, R> {
    id: Option<u32>,
    payload: Vec<u8>,
    phantom_d: core::marker::PhantomData<D>,
    phantom_m: core::marker::PhantomData<M>,
    phantom_r: core::marker::PhantomData<R>,
}

impl<'a, D, M, R> RpcClientRequest<D, M, R>
where
    D: dataformat::DataFormat,
    M: Serialize + Deserialize<'a>,
    R: Serialize + Deserialize<'a>,
{
    pub fn new(id: Option<u32>, payload: Vec<u8>) -> Self {
        Self {
            id,
            payload,
            phantom_d: core::marker::PhantomData,
            phantom_m: core::marker::PhantomData,
            phantom_r: core::marker::PhantomData,
        }
    }
    pub fn payload(&self) -> &[u8] {
        &self.payload
    }
    pub fn take_payload(&mut self) -> Vec<u8> {
        mem::take(&mut self.payload)
    }
    pub fn handle_response(&self, response_payload: &'a [u8]) -> RpcResult<R> {
        let Some(id) = self.id else {
            return Err(RpcError {
                kind: RpcErrorKind::InvalidRequest,
                message: Some("request ID is missing".to_owned()),
            });
        };
        match D::unpack::<Response<R>>(response_payload) {
            Ok(r) => {
                let (res_id, res) = r.into_parts();
                if res_id != id {
                    return Err(RpcError {
                        kind: RpcErrorKind::InvalidRequest,
                        message: Some("response ID does not match request ID".to_owned()),
                    });
                }
                res.into()
            }
            Err(e) => Err(RpcError {
                kind: RpcErrorKind::ParseError,
                message: Some(e.to_string()),
            }),
        }
    }
}
