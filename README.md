<h2>
  RoboPLC RPC - JSON RPC 2.0 minimalistic server and client
  <a href="https://crates.io/crates/roboplc-rpc"><img alt="crates.io page" src="https://img.shields.io/crates/v/roboplc-rpc.svg"></img></a>
  <a href="https://docs.rs/roboplc-rpc"><img alt="docs.rs page" src="https://docs.rs/roboplc-rpc/badge.svg"></img></a>
  <a href="https://github.com/roboplc/roboplc-rpc/actions/workflows/ci.yml">
    <img alt="GitHub Actions CI" src="https://github.com/roboplc/roboplc-rpc/actions/workflows/ci.yml/badge.svg"></img>
  </a>
</h2>

JSON RPC 2.0 server and client, part of [RoboPLC](https://www.roboplc.com)
project.

## Description

Ultra-lightweight JSON RPC 2.0 server and client library. Fully generic
implementation, no runtime dispatching, maximum performance and exhaustiveness.

Protocol-agnostic, can be used with any transport layer.

Note: batch requests are not supported as used pretty never in practice. In
case if you `really` need them, submit an issue/pull request.

## Example

### Client

```rust
use serde::{Serialize, Deserialize};
use roboplc_rpc::{client::RpcClient, dataformat};

#[derive(Serialize, Deserialize)]
// use method/params for the canonical JSON-RPC 2.0
#[serde(tag = "m", content = "p", rename_all = "lowercase", deny_unknown_fields)]
enum MyMethod<'a> {
    Test {},
    Hello { name: &'a str },
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
enum MyResult {
    General { ok: bool },
    String(String),
}

let client: RpcClient<dataformat::Json, MyMethod, MyResult> = RpcClient::new();
let req = client.request(MyMethod::Hello { name: "world" }).unwrap();
// send req.payload() via the chosen transport to the server
// if response is received, get the result
// let result = client.handle_response(&response); // returns MyResult or RpcError
```

### Server

```rust
use serde::{Serialize, Deserialize};
use roboplc_rpc::{RpcResult, dataformat, server::{RpcServer, RpcServerHandler}};

// use the same types as in the client, e.g. share a common crate
#[derive(Serialize, Deserialize)]
#[serde(tag = "m", content = "p", rename_all = "lowercase", deny_unknown_fields)]
enum MyMethod<'a> {
    Test {},
    Hello { name: &'a str },
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
enum MyResult {
    General { ok: bool },
    String(String),
}

struct MyRpc {}

impl<'a> RpcServerHandler<'a> for MyRpc {
    type Method = MyMethod<'a>;
    type Result = MyResult;
    type Source = std::net::IpAddr;

    fn handle_call(&self, method: MyMethod, source: Self::Source)
        -> RpcResult<MyResult> {
        println!("Received call from {}", source);
        match method {
            MyMethod::Test {} => Ok(MyResult::General { ok: true }),
            MyMethod::Hello { name } => Ok(MyResult::String(format!("Hello, {}", name))),
        }
    }
}

let server = roboplc_rpc::server::RpcServer::new(MyRpc {});
// get the request from the transport
let request_payload = r#"{"i":1,"m":"hello","p":{"name":"world"}}"#.as_bytes();
if let Some(response_payload) = server.handle_request_payload::<dataformat::Json>(
    request_payload, std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST)) {
    // send response_payload via the chosen transport to the client. if the
    // payload is none, either the client does not need a response or the
    // request was completely invalid (no error can be returned)
}
```

## Canonical/minimalistic JSON-RPC 2.0

By default the crate works in a "minimalistic" mode:

* `jsonrpc` field is not required in the request/response, the version is never
  checked.

* `id`, `result` and `error` fields are renamed to `i`, `r` and `e` respectively.

The mode can be changed to JSON-RPC 2.0 canonical by enabling the `canonical`
feature.

## Features

* `std` - std support (enabled by default).
* `msgpack` - enables MessagePack serialization support.
* `http` - certain tools for HTTP transport (calls via HTTP GET, minimalistic responses).
* `canonical` - enable canonical JSON-RPC 2.0

## no-std

This library is `no_std` compatible. Use `--no-default-features` to disable `std` support.

[heapless::String](https://docs.rs/heapless/latest/heapless/struct.String.html)
is used for strings instead of the standard one (for error messages).

Limitations:

* Request id can be `u32` only.
* Provides data types only, no client/server implementations.
* Error messages can be 128 bytes long only.


## MSRV

1.68.0
