#![cfg_attr(not(feature = "std"), no_std)]

use serde::{Deserialize, Serialize};

const JSONRPC_VERSION: &str = "2.0";

#[cfg(feature = "canonical")]
const VERSION_HEADER: Option<()> = Some(());
#[cfg(not(feature = "canonical"))]
const VERSION_HEADER: Option<()> = None;

const ERR_INVALID_PROTOCOL_VERSION: &str = "Invalid protocol version";

#[cfg(feature = "std")]
type Id = serde_json::Value;
#[cfg(not(feature = "std"))]
type Id = u32;

#[cfg(feature = "std")]
type String = std::string::String;
#[cfg(not(feature = "std"))]
type String = heapless::String<128>;

#[cfg(feature = "std")]
pub mod client;
#[cfg(feature = "std")]
pub mod dataformat;
pub mod request;
pub mod response;
#[cfg(feature = "std")]
pub mod server;
pub mod tools;

fn de_validate_version<'de, D>(deserializer: D) -> Result<Option<()>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let version: Option<&str> = Deserialize::deserialize(deserializer)?;
    #[cfg(feature = "canonical")]
    if version.map_or(false, |v| v != JSONRPC_VERSION) {
        return Err(serde::de::Error::custom(ERR_INVALID_PROTOCOL_VERSION));
    }
    Ok(version.map(|_| ()))
}

#[allow(clippy::trivially_copy_pass_by_ref, clippy::ref_option)]
fn serialize_version<S>(_: &Option<()>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    JSONRPC_VERSION.serialize(serializer)
}

const RPC_ERROR_PARSE_ERROR: i16 = -32700;
const RPC_ERROR_INVALID_REQUEST: i16 = -32600;
const RPC_ERROR_METHOD_NOT_FOUND: i16 = -32601;
const RPC_ERROR_INVALID_PARAMS: i16 = -32602;
const RPC_ERROR_INTERNAL_ERROR: i16 = -32603;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RpcErrorKind {
    ParseError,
    InvalidRequest,
    MethodNotFound,
    InvalidParams,
    InternalError,
    Custom(i16),
}

#[cfg(feature = "std")]
impl core::fmt::Display for RpcErrorKind {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", i16::from(*self))
    }
}

impl From<i16> for RpcErrorKind {
    fn from(code: i16) -> Self {
        match code {
            RPC_ERROR_PARSE_ERROR => RpcErrorKind::ParseError,
            RPC_ERROR_INVALID_REQUEST => RpcErrorKind::InvalidRequest,
            RPC_ERROR_METHOD_NOT_FOUND => RpcErrorKind::MethodNotFound,
            RPC_ERROR_INVALID_PARAMS => RpcErrorKind::InvalidParams,
            RPC_ERROR_INTERNAL_ERROR => RpcErrorKind::InternalError,
            _ => RpcErrorKind::Custom(code),
        }
    }
}

impl From<RpcErrorKind> for i16 {
    fn from(code: RpcErrorKind) -> Self {
        match code {
            RpcErrorKind::ParseError => RPC_ERROR_PARSE_ERROR,
            RpcErrorKind::InvalidRequest => RPC_ERROR_INVALID_REQUEST,
            RpcErrorKind::MethodNotFound => RPC_ERROR_METHOD_NOT_FOUND,
            RpcErrorKind::InvalidParams => RPC_ERROR_INVALID_PARAMS,
            RpcErrorKind::InternalError => RPC_ERROR_INTERNAL_ERROR,
            RpcErrorKind::Custom(code) => code,
        }
    }
}

impl Serialize for RpcErrorKind {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        i16::from(*self).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for RpcErrorKind {
    fn deserialize<D>(deserializer: D) -> Result<RpcErrorKind, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        i16::deserialize(deserializer).map(RpcErrorKind::from)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RpcError {
    #[serde(rename = "code")]
    kind: RpcErrorKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
}

impl RpcError {
    pub fn new0(kind: RpcErrorKind) -> Self {
        Self {
            kind,
            message: None,
        }
    }
    pub fn new(kind: RpcErrorKind, message: String) -> Self {
        Self {
            kind,
            message: Some(message),
        }
    }
    pub fn kind(&self) -> RpcErrorKind {
        self.kind
    }
    pub fn message(&self) -> Option<&str> {
        self.message.as_deref()
    }
}

#[cfg(feature = "std")]
impl core::fmt::Display for RpcError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if let Some(message) = &self.message {
            write!(f, "{} ({})", message, self.kind)
        } else {
            write!(f, "{}", self.kind)
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for RpcError {}

pub type RpcResult<R> = Result<R, RpcError>;

// ************************************* my

//#[derive(Serialize, Deserialize, Debug)]
//#[serde(tag = "method", content = "params", deny_unknown_fields)]
//enum MyMethod {
//#[serde(rename = "test")]
//Test {},
//#[serde(rename = "hello")]
//Hello { name: String },
//#[serde(rename = "list")]
//List { i: String },
//#[serde(rename = "complicated")]
//Complicated {},
//}

//#[derive(Debug, Serialize, Deserialize)]
//#[serde(untagged)]
//enum MyHandlerResult {
//General { ok: bool },
//String(String),
//}

//struct MyRpc {}

//impl server::RpcServer<'_> for MyRpc {
//type Method = MyMethod;
//type Result = MyHandlerResult;
//type Source = &'static str;

//fn rpc_handler(
//&self,
//method: MyMethod,
//_source: Self::Source,
//_response_required: bool,
//) -> RpcResult<MyHandlerResult> {
//match method {
//MyMethod::Test {} => Ok(MyHandlerResult::General { ok: true }),
//MyMethod::Hello { name } => Ok(MyHandlerResult::String(format!("Hello, {}", name))),
//MyMethod::List { i } => Ok(MyHandlerResult::String(format!("List, {}", i))),
//MyMethod::Complicated {} => Err(RpcError {
//kind: RpcErrorKind::Custom(-32000),
//message: Some("Complicated method not implemented".into()),
//}),
//}
//}
//}

//fn main() {
//env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
//let myrpc = MyRpc {};
//let call = MyMethod::Hello {
//name: "world".to_owned(),
//};
//let call = MyMethod::Complicated {};
//let qs: tools::http::QueryString = Request::new(1.into(), call).try_into().unwrap();
//println!("{}", qs);
//let req: Request<MyMethod> = qs.try_into().unwrap();
//dbg!(&req);
//let response = myrpc.handle_request(req, "local").unwrap();
//let h: tools::http::HttpResponse = response.try_into().unwrap();
//dbg!(h);
//let (status, headers, body) = h.into_parts();
//println!("{:?} {:?}", status, headers);
//println!("{}", std::str::from_utf8(&body).unwrap());
//let client: RpcClient<dataformat::Json, MyMethod, MyHandlerResult> = RpcClient::new();
//let req = client.request(MyMethod::Test {}).unwrap();
//if let Some(v) = myrpc.handle_request_payload::<dataformat::Json, _>(req.payload(), "local") {
//dbg!(req.handle_response(v.as_slice())).ok();
//}
//let req = client.request(MyMethod::Hello { name: "world" }).unwrap();
//if let Some(v) = myrpc.handle_request_payload::<dataformat::Json, _>(req.payload(), "local") {
//dbg!(req.handle_response(v.as_slice())).ok();
//}
//let req = client.request(MyMethod::Complicated {}).unwrap();
//if let Some(v) = myrpc.handle_request_payload::<dataformat::Json, _>(req.payload(), "local") {
//dbg!(req.handle_response(v.as_slice())).ok();
//}
//let invalid_params_req = r#"{"jsonrpc":"2.0","id":3,"method":"test","params":{"abc": 123}}"#;
//let resp =
//myrpc.handle_request_payload::<dataformat::Json, _>(invalid_params_req.as_bytes(), "local");
//if let Some(v) = resp {
//dbg!(std::str::from_utf8(v.as_slice())).ok();
//}
//}
