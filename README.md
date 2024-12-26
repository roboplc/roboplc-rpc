# RoboPLC RPC

JSON RPC 2.0 server and client, part of [RoboPLC](https://www.roboplc.com)
project.

## Description

Ultra-lightweight JSON RPC 2.0 server and client library. Fully generic
implementation, no runtime dispatching, maximum performance and exhaustiveness.

Protocol-agnostic, can be used with any transport layer.

Note: batch requests are not supported as used pretty never in practice. In
case if you `really` need them, submit an issue/pull request.

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

Limitations:

* Request id can be `u32` only.
* Provides data types only, no client/server implementations.
* Error messages can be 128 bytes long only.


## MSRV

1.68.0
