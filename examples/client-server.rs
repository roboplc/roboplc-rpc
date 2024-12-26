use roboplc_rpc::{
    client::RpcClient,
    dataformat,
    server::{self, RpcServer as _},
    RpcError, RpcErrorKind, RpcResult,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(
    feature = "canonical",
    serde(tag = "method", content = "params", deny_unknown_fields)
)]
#[cfg_attr(
    not(feature = "canonical"),
    serde(tag = "m", content = "p", deny_unknown_fields)
)]
enum MyMethod<'a> {
    #[serde(rename = "test")]
    Test {},
    #[serde(rename = "hello")]
    Hello { name: &'a str },
    #[serde(rename = "list")]
    List { i: &'a str },
    #[serde(rename = "complicated")]
    Complicated {},
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum MyResult {
    General { ok: bool },
    String(String),
}

struct MyRpc {}

impl<'a> server::RpcServer<'a> for MyRpc {
    type Method = MyMethod<'a>;
    type Result = MyResult;
    type Source = &'static str;

    fn rpc_handler(&self, method: MyMethod, _source: Self::Source) -> RpcResult<MyResult> {
        match method {
            MyMethod::Test {} => Ok(MyResult::General { ok: true }),
            MyMethod::Hello { name } => Ok(MyResult::String(format!("Hello, {}", name))),
            MyMethod::List { i } => Ok(MyResult::String(format!("List, {}", i))),
            MyMethod::Complicated {} => Err(RpcError::new(
                RpcErrorKind::Custom(-32000),
                "Complicated method not implemented".into(),
            )),
        }
    }
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let myrpc = MyRpc {};
    let client: RpcClient<dataformat::Json, MyMethod, MyResult> = RpcClient::new();
    let req = client.request(MyMethod::Test {}).unwrap();
    println!(
        "request payload: {}",
        std::str::from_utf8(req.payload()).unwrap()
    );
    if let Some(v) = myrpc.handle_request_payload::<dataformat::Json>(req.payload(), "local") {
        println!("response: {}", std::str::from_utf8(v.as_slice()).unwrap());
        dbg!(req.handle_response(v.as_slice())).ok();
    }
    println!(
        "request payload: {}",
        std::str::from_utf8(req.payload()).unwrap()
    );
    let req = client.request(MyMethod::Hello { name: "world" }).unwrap();
    if let Some(v) = myrpc.handle_request_payload::<dataformat::Json>(req.payload(), "local") {
        println!("response: {}", std::str::from_utf8(v.as_slice()).unwrap());
        dbg!(req.handle_response(v.as_slice())).ok();
    }
    println!(
        "request payload: {}",
        std::str::from_utf8(req.payload()).unwrap()
    );
    let req = client.request(MyMethod::Complicated {}).unwrap();
    if let Some(v) = myrpc.handle_request_payload::<dataformat::Json>(req.payload(), "local") {
        println!("response: {}", std::str::from_utf8(v.as_slice()).unwrap());
        dbg!(req.handle_response(v.as_slice())).ok();
    }
    let invalid_params_req = r#"{"jsonrpc":"2.0","id":3,"method":"test","params":{"abc": 123}}"#;
    println!("request payload: {}", invalid_params_req);
    let resp =
        myrpc.handle_request_payload::<dataformat::Json>(invalid_params_req.as_bytes(), "local");
    if let Some(v) = resp {
        println!("response: {}", std::str::from_utf8(v.as_slice()).unwrap());
        dbg!(std::str::from_utf8(v.as_slice())).ok();
    }
}
