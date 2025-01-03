use core::fmt;
use std::collections::BTreeMap;

use http::{header, StatusCode};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::{json, Value};

/// HTTP tools error type
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Serialization error
    #[error("pack error: {0}")]
    Serialization(#[from] serde_json::Error),
    /// Invalid data
    #[error("invalid data: {0}")]
    InvalidData(String),
}

use crate::{request::Request, response::Response};

/// Query string representation of a JSON-RPC request,
/// as: `i=1&m=method&param1=value1&param2=value2`, where id is optional
///
/// Booleans ("true"/"false"), numbers and "null" are parsed automatically,
#[derive(Debug)]
pub struct QueryString(String);

impl QueryString {
    /// Create a new query string from a string
    pub fn new(s: &str) -> Self {
        QueryString(s.to_owned())
    }
}

impl From<String> for QueryString {
    fn from(s: String) -> Self {
        QueryString(s)
    }
}

impl fmt::Display for QueryString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<QueryString> for String {
    fn from(qs: QueryString) -> Self {
        qs.0
    }
}

impl AsRef<str> for QueryString {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl<M: Serialize> TryFrom<Request<M>> for QueryString {
    type Error = Error;

    fn try_from(req: Request<M>) -> Result<Self, Self::Error> {
        request_into_query_string(&req).map(QueryString)
    }
}

impl<M: DeserializeOwned + Serialize> TryFrom<QueryString> for Request<M> {
    type Error = Error;

    fn try_from(qs: QueryString) -> Result<Self, Self::Error> {
        request_from_query_string(&qs.0)
    }
}

fn parse_string(s: &str) -> Value {
    if s == "true" {
        Value::Bool(true)
    } else if s == "false" {
        Value::Bool(false)
    } else if s == "null" {
        Value::Null
    } else if let Ok(n) = s.parse::<u64>() {
        Value::Number(n.into())
    } else if let Ok(n) = s.parse::<i64>() {
        Value::Number(n.into())
    } else if let Ok(n) = s.parse::<f64>() {
        Value::Number(serde_json::value::Number::from_f64(n).unwrap())
    } else {
        Value::String(s.to_string())
    }
}

fn request_from_query_string<M: DeserializeOwned + Serialize>(
    qs: &str,
) -> Result<Request<M>, Error> {
    let mut id: Option<Value> = None;
    let mut method: Option<String> = None;
    let mut params: BTreeMap<String, Value> = BTreeMap::new();
    for (i, (name, value)) in url::form_urlencoded::parse(qs.as_bytes())
        .into_iter()
        .enumerate()
    {
        match name.as_ref() {
            "i" if i == 0 => {
                id = Some(serde_json::from_str(&value)?);
            }
            "m" if method.is_none() => {
                method = Some(value.to_string());
            }
            n => {
                params.insert(n.to_string(), parse_string(&value));
            }
        }
    }
    let method_name = method.ok_or(Error::InvalidData("the method is missing".into()))?;
    #[cfg(feature = "canonical")]
    let method = serde_json::from_value(json!({
        "method": method_name,
        "params": params,
    }))?;
    #[cfg(not(feature = "canonical"))]
    let method = serde_json::from_value(json!({
        "m": method_name,
        "p": params,
    }))?;
    if let Some(id) = id {
        Ok(Request::new(id, method))
    } else {
        Ok(Request::new0(method))
    }
}

fn value_to_string(field: &str, value: &Value) -> Result<String, Error> {
    Ok(match value {
        Value::Null => "null".to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => s.to_string(),
        _ => {
            return Err(Error::InvalidData(format!(
                "unsupported value type for field '{}'",
                field
            )))
        }
    })
}

fn request_into_query_string<M: Serialize>(req: &Request<M>) -> Result<String, Error> {
    let mut pairs = Vec::new();
    if let Some(id) = &req.id {
        pairs.push(("i", id.to_string()));
    }
    let req_value = serde_json::to_value(&req.method)?;
    let req_map = req_value
        .as_object()
        .ok_or(Error::InvalidData("invalid request".into()))?;
    let method = req_map
        .get("method")
        .ok_or(Error::InvalidData("method is missing".into()))?;
    pairs.push((
        "m",
        method
            .as_str()
            .ok_or(Error::InvalidData("invalid method name".into()))?
            .to_string(),
    ));
    if let Some(params) = req_map.get("params") {
        let params = params
            .as_object()
            .ok_or(Error::InvalidData("params must be object".into()))?;
        for (name, value) in params {
            pairs.push((name, value_to_string(name, value)?));
        }
    }
    Ok(url::form_urlencoded::Serializer::new(String::new())
        .extend_pairs(pairs)
        .finish())
}

#[derive(Debug)]
#[allow(clippy::module_name_repetitions)]
/// A minimalistic HTTP response (no JSON RPC version, call id is placed to `X-JSONRPC-ID` header)
pub struct HttpResponse {
    status: http::StatusCode,
    headers: http::header::HeaderMap,
    body: String,
}

impl HttpResponse {
    /// HTTP status code (200 for success, 500 for error)
    pub fn status(&self) -> http::StatusCode {
        self.status
    }
    /// HTTP headers
    pub fn headers(&self) -> &http::header::HeaderMap {
        &self.headers
    }
    /// HTTP body
    pub fn body(&self) -> &str {
        &self.body
    }
    /// Mutable reference to HTTP headers
    pub fn headers_mut(&mut self) -> &mut http::header::HeaderMap {
        &mut self.headers
    }
    /// Split the response into parts
    pub fn into_parts(self) -> (http::StatusCode, http::header::HeaderMap, String) {
        (self.status, self.headers, self.body)
    }
}

impl<R: Serialize> TryFrom<Response<R>> for HttpResponse {
    type Error = Error;

    fn try_from(response: Response<R>) -> Result<Self, Self::Error> {
        let (id, res) = response.into_parts();
        let status = if res.is_ok() {
            StatusCode::OK
        } else {
            StatusCode::INTERNAL_SERVER_ERROR
        };
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/json"),
        );
        headers.insert(
            "X-JSONRPC-ID",
            value_to_string("", &id)?.parse().map_err(|e| {
                Error::InvalidData(format!("failed to parse id as http header: {}", e))
            })?,
        );
        Ok(HttpResponse {
            status,
            headers,
            body: serde_json::to_string(&res)?,
        })
    }
}
