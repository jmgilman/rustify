# rustify

<p align="center">
    <a href="https://github.com/jmgilman/rustify/actions/workflows/validate.yml">
        <img src="https://github.com/jmgilman/rustify/actions/workflows/validate.yml/badge.svg"/>
    </a>
    <a href="https://crates.io/crates/rustify">
        <img src="https://img.shields.io/crates/v/rustify">
    </a>
    <a href="https://docs.rs/rustify">
        <img src="https://img.shields.io/docsrs/rustify" />
    </a>
</p>

> A Rust crate which provides an abstraction layer over HTTP REST API endpoints

Rustify is a small crate which provides a way to easily scaffold code which
communicates with HTTP REST API endpoints. It covers simple cases such as basic
GET requests as well as more advanced cases such as sending serialized data
and deserializing the result. A derive macro is provided to keep code DRY.

Rustify provides both a trait for implementing API endpoints as well as clients
for executing requests against the defined endpoints. This crate targets `async`
first, however, blocking clients can be found in `rustify::blocking::clients`.
Additionally, the `Endpoint` trait offers both `async` and blocking variants
of each execution method.

Presently, rustify only supports JSON serialization and generally assumes the
remote endpoint accepts and responds with JSON. Raw byte data can be sent
by tagging a field with `#[endpoint(data)]` and can be received by using the
`Endpoint::exec_raw()` method. 

## Installation

```
cargo add rustify
```

## Architecture

This crate consists of two primary traits:

* The `Endpoint` trait which represents a remote HTTP REST API endpoint
* The `Client` trait which is responsible for executing the `Endpoint`

This provides a loosely coupled interface that allows for multiple
implementations of the `Client` trait which may use different HTTP backends. The 
`Client` trait in particular was kept intentionally easy to implement and is
only required to send `http::Request`s and return `http::Response`s. A blocking
variant of the client (`rustify::blocking::client::Client`) is provided for 
implementations that block. 

The `Endpoint` trait is what will be most implemented by end-users of this
crate. Since the implementation can be verbose and most functionality can be
defined with very little syntax, a macro is provided via `rustify_derive` which
should be used for generating implementations of this trait. 

## Usage

The below example creates a `Test` endpoint that, when executed, will send a GET
request to `http://api.com/test/path` and expect an empty response:

```rust
use rustify::clients::reqwest::Client;
use rustify::endpoint::Endpoint;
use rustify_derive::Endpoint;
use serde::Serialize;

#[derive(Debug, Endpoint, Serialize)]
#[endpoint(path = "test/path")]
struct Test {}

let endpoint = Test {};
let client = Client::default("http://api.com");
let result = endpoint.exec(&client);
assert!(result.is_ok());
```

## Advanced Usage

This examples demonstrates the complexity available using the full suite of
options offered by the macro:

```rust
use bytes::Bytes;
use derive_builder::Builder;
use rustify::clients::reqwest::Client;
use rustify::endpoint::{Endpoint, MiddleWare};
use rustify::errors::ClientError;
use rustify_derive::Endpoint;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::skip_serializing_none;

struct Middle {}
impl MiddleWare for Middle {
    fn request<E: Endpoint>(
        &self,
        _: &E,
        req: &mut http::Request<Vec<u8>>,
    ) -> Result<(), ClientError> {
        req.headers_mut()
            .append("X-API-Token", http::HeaderValue::from_static("mytoken"));
        Ok(())
    }
    fn response<E: Endpoint>(
        &self,
        _: &E,
        resp: &mut http::Response<Bytes>,
    ) -> Result<(), ClientError> {
        let resp_body = resp.body().clone();
        let wrapper: TestWrapper =
            serde_json::from_slice(&resp_body).map_err(|e| ClientError::ResponseParseError {
                source: Box::new(e),
                content: String::from_utf8(resp_body.to_vec()).ok(),
            })?;
        let data = wrapper.result.to_string();
        *resp.body_mut() = bytes::Bytes::from(data);
        Ok(())
    }
}

#[derive(Deserialize)]
struct TestResponse {
    age: u8,
}

#[derive(Deserialize)]
struct TestWrapper {
    result: Value,
}

#[skip_serializing_none]
#[derive(Builder, Debug, Default, Endpoint, Serialize)]
#[endpoint(
    path = "test/path/{self.name}",
    method = "POST",
    result = "TestResponse",
    builder = "true"
)]
#[builder(setter(into, strip_option), default)]
struct Test {
    #[serde(skip)]
    name: String,
    kind: String,
    special: Option<bool>,
    optional: Option<String>,
}

let client = Client::default("http://api.com");
let endpoint = Test::builder()
    .name("test")
    .kind("test")
    .build()
    .unwrap();
let result = endpoint.exec_mut(&client, &Middle {});
```

Breaking this down:

```rust
    #[endpoint(
        path = "test/path/{self.name}",
        method = "POST",
        result = "TestResponse",
        builder = "true"
    )]
```

* The `path` argument supports basic substitution using curly braces. In this 
case the final url would be `http://api.com/test/path/test`. Since the `name` 
field is only used to build the endpoint URL, we add the `#[serde(skip)]` 
attribute to inform `serde` to not serialize this field when building the 
request.
* The `method` argument specifies the type of the HTTP request. 
* The `result` argument specifies the type of response that the `exec()` 
method will return. This type must derive `Deserialize`. 
* The `builder` argument tells the macro to add some useful functions for when
the endpoint is using the `Builder` derive macro from [derive_builder][1]. In
particular, it adds a `builder()` static method to the base struct which returns
a default instance of the builder for the endpoint.

Endpoints contain various methods for executing requests; in this example the
`exec_mut()` variant is being used which allows passing an instance of an 
object that implements `MiddleWare` which can be used to mutate the request and
response object respectively. Here an arbitrary request header containing a
fictitious API token is being injected and the response has a wrapper removed
before final parsing.  

This example also demonstrates a common pattern of using 
[skip_serializing_none][2] macro to force `serde` to not serialize fields of 
type `Option::None`. When combined with the `default` parameter offered by 
[derive_builder][1] the result is an endpoint which can have required and/or 
optional fields as needed and which don't get serialized if absent when 
building. For example:

```rust
// Errors, `kind` field is required
let endpoint = Test::builder()
    .name("test")
    .build()
    .unwrap();

// Produces POST http://api.com/test/path/test {"kind": "test"}
let endpoint = Test::builder()
    .name("test")
    .kind("test")
    .build()
    .unwrap();

// Produces POST http://api.com/test/path/test {"kind": "test", "optional": "yes"}
let endpoint = Test::builder()
    .name("test")
    .kind("test")
    .optional("yes")
    .build()
    .unwrap()
```

## Error Handling

All errors generated by this crate are wrapped in the `ClientError` enum
provided by the crate.

## Testing

See the the [tests](tests) directory for tests. Run tests with
`cargo test`. 

## Contributing

1. Fork it (https://github.com/jmgilman/rustify/fork)
2. Create your feature branch (git checkout -b feature/fooBar)
3. Commit your changes (git commit -am 'Add some fooBar')
4. Push to the branch (git push origin feature/fooBar)
5. Create a new Pull Request

[1]: https://docs.rs/derive_builder/latest/derive_builder/
[2]: https://docs.rs/serde_with/1.9.4/serde_with/attr.skip_serializing_none.html