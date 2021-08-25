//! Rustify is a small crate which provides a way to easily scaffold code which
//! communicates with HTTP REST API endpoints. It covers simple cases such as basic
//! GET requests as well as more advanced cases such as sending serialized data
//! and deserializing the result. A derive macro is provided to keep code DRY.
//!
//! Rustify provides both a trait for implementing API endpoints as well as clients
//! for executing requests against the defined endpoints. Currently, only a client
//! using [reqwest::blocking][1] is provided.
//!
//! Presently, rustify only supports JSON serialization and generally assumes the
//! remote endpoint accepts and responds with JSON.
//!
//!
//! ## Architecture
//!
//! This crate consists of two primary traits:
//!
//! * The [Endpoint][crate::endpoint::Endpoint] trait which represents a remote
//!   HTTP REST API endpoint
//! * The [Client][crate::client::Client] trait which is responsible for
//!   executing the Endpoint
//!
//! This provides a loosely coupled interface that allows for multiple
//! implementations of the Client trait which may use different HTTP backends.
//! The Client trait in particular was kept intentionally easy to implement and
//! is only required to send a HTTP [request][crate::client::Request] consisting
//! of a URL, method, and body and then return the
//! [response][crate::client::Response] consisting of a URL, response code, and
//! response body. The crate currently only provides a blocking client based on
//! the [reqwest][2] crate.
//!
//! The Endpoint trait is what will be most implemented by end-users of this
//! crate. Since the implementation can be verbose and most functionality can be
//! defined with very little syntax, a macro is provided via `rustify_derive`
//! which should be used for generating implementations of this trait.
//!
//!
//! ## Usage
//!
//! The below example creates a `Test` endpoint that, when executed, will send a
//! GET request to `http://!api.com/test/path` and expect an empty response:
//!
//! ```
//! use rustify::clients::reqwest::ReqwestClient;
//! use rustify::endpoint::Endpoint;
//! use rustify_derive::Endpoint;
//! use serde::Serialize;
//!
//! #[derive(Debug, Endpoint, Serialize)]
//! #[endpoint(path = "test/path")]
//! struct Test {}
//!
//! let endpoint = Test {};
//! let client = ReqwestClient::default("http://!api.com");
//! let result = endpoint.exec(&client);
//! ```
//!
//! ## Advanced Usage
//!
//! This examples demonstrates the complexity available using the full suite of
//! options offered by the macro:
//!
//! ```rust
//! use derive_builder::Builder;
//! use rustify::clients::reqwest::ReqwestClient;
//! use rustify::{endpoint::{Endpoint, MiddleWare}, errors::ClientError};
//! use rustify_derive::Endpoint;
//! use serde::{Deserialize, Serialize};
//! use serde_json::Value;
//! use serde_with::skip_serializing_none;
//!
//! struct Middle {}
//! impl MiddleWare for Middle {
//!     fn request<E: Endpoint>(
//!         &self,
//!         _: &E,
//!         req: &mut rustify::client::Request,
//!     ) -> Result<(), ClientError> {
//!         req.headers
//!             .push(("X-API-Token".to_string(), "mytoken".to_string()));
//!         Ok(())
//!     }
//!     fn response<E: Endpoint>(
//!         &self,
//!         _: &E,
//!         resp: &mut rustify::client::Response,
//!     ) -> Result<(), ClientError> {
//!         let err_body = resp.body.clone();
//!         let wrapper: TestWrapper =
//!             serde_json::from_slice(&resp.body).map_err(|e| ClientError::ResponseParseError {
//!                 source: Box::new(e),
//!                 content: String::from_utf8(err_body).ok(),
//!             })?;
//!         resp.body = wrapper.result.to_string().as_bytes().to_vec();
//!         Ok(())
//!     }
//! }
//!
//! #[derive(Deserialize)]
//! struct TestResponse {
//!     age: u8,
//! }
//!
//! #[derive(Deserialize)]
//! struct TestWrapper {
//!     result: Value,
//! }
//!
//! fn test_complex() {
//!     #[skip_serializing_none]
//!     #[derive(Builder, Debug, Default, Endpoint, Serialize)]
//!     #[endpoint(
//!         path = "test/path/{self.name}",
//!         method = "POST",
//!         result = "TestResponse",
//!         builder = "true"
//!     )]
//!     #[builder(setter(into, strip_option), default)]
//!     struct Test {
//!         #[serde(skip)]
//!         name: String,
//!         kind: String,
//!         special: Option<bool>,
//!         optional: Option<String>,
//!     }
//!
//!     let client = ReqwestClient::default("http://!api.com");
//!     let result = Test::builder().name("test").kind("test").exec_mut(&client, &Middle {});
//!
//! }
//! ```
//!
//! Breaking this down:
//!
//! ```ignore
//!     #[endpoint(
//!         path = "test/path/{self.name}",
//!         method = "POST",
//!         result = "TestResponse",
//!         builder = "true"
//!     )]
//!
//! ```
//!
//! * The `path` argument supports basic substitution using curly braces. In
//!   this case the final url would be `http://!api.com/test/path/test`. Since
//!   the `name` field is only used to build the endpoint URL, we add the
//!   `#[serde(skip)]` attribute to inform `serde` to not serialize this field
//!   when building the request.
//! * The `method` argument specifies the type of the HTTP request.
//! * The `result` argument specifies the type of response that the
//!   [exec()][crate::endpoint::Endpoint::execute] method will return. This
//!   type must derive [serde::Deserialize].
//! * The `builder` argument tells the macro to add some useful functions for
//!   when the endpoint is using the `Builder` derive macro from
//!   [derive_builder][3]. In particular, it adds a `builder()` static method to
//!   the base struct and the `exec()` methods to the generated `TestBuilder`
//!   struct which automatically calls `build()` on `TestBuilder` and then
//!   executes the result. This allows for concise calls like this:
//!   `Test::builder().name("test").kind("test").exec(&client);`
//!
//! Endpoints contain two methods for executing requests; in this example the
//! `execute_m()` variant is being used which allows passing an instance of an
//! object that implements `MiddleWare` which can be used to mutate the request
//! and response object respectively. Here the an arbitrary request header
//! containing a fictitious API token is being injected and the response has a
//! wrapper removed before final parsing.
//!
//! This example also demonstrates a common pattern of using
//! [skip_serializing_none][4] macro to force `serde` to not serialize fields of
//! type `Option::None`. When combined with the `default` parameter offered by
//! [derive_builder][3] the result is an endpoint which can have required and/or
//! optional fields as needed and which don't get serialized when not specified
//! when building. For example:
//!
//! ```ignore
//! // Errors, `kind` field is required
//! let result = Test::builder().name("test").exec(&client);
//!
//! // Produces POST http://!api.com/test/path/test {"kind": "test"}
//! let result = Test::builder().name("test").kind("test").exec(&client);
//!
//! // Produces POST http://!api.com/test/path/test {"kind": "test", "optional": "yes"}
//! let result = Test::builder().name("test").kind("test").optional("yes").exec&client);
//! ```
//!
//! ## Error Handling
//!
//! All errors generated by this crate are wrapped in the
//! [ClientError][crate::errors::ClientError] enum provided by the crate.
//!
//! [1]: https://docs.rs/reqwest/latest/reqwest/blocking/index.html
//! [2]: https://docs.rs/reqwest/latest/reqwest/index.html
//! [3]: https://docs.rs/derive_builder/latest/derive_builder/
//! [4]: https://docs.rs/serde_with/1.9.4/serde_with/attr.skip_serializing_none.html

pub mod blocking;
pub mod client;
pub mod clients;
pub mod endpoint;
pub mod enums;
pub mod errors;
