//! # rustify
//!
//! <p align="center">
//!     <a href="https://crates.io/crates/rustify">
//!         <img src="https://img.shields.io/crates/v/rustify">
//!     </a>
//!     <a href="https://docs.rs/rustify">
//!         <img src="https://img.shields.io/docsrs/rustify" />
//!     </a>
//!     <a href="https://github.com/jmgilman/rustify/actions/workflows/ci.yml">
//!         <img src="https://github.com/jmgilman/rustify/actions/workflows/ci.yml/badge.svg"/>
//!     </a>
//! </p>
//!
//! > A Rust library for interacting with HTTP API endpoints
//!
//! Rustify is a small library written in Rust which eases the burden of
//! scaffolding HTTP APIs. It provides an `Endpoint` trait along with a macro helper
//! which allows templating various remote endpoints. Both asynchronous and
//! synchrounous clients are offered for executing requests against endpoints with
//! the option of implementing custom clients using the `Client` trait.
//!
//! Rustify provides support for serializing requests and deserializing responses.
//! Raw requests and responses in the form of bytes are also supported. The library
//! also contains many helpers for dealing with requests like support for middleware
//! and wrapping API responses.
//!
//! ## Installation
//!
//! Add rustify as a dependency to your cargo.toml:
//!
//! ```ignore
//! [dependencies]
//! rustify = "0.5.2"
//! rustify_derive = "0.5.2"
//! ```
//!
//! ## Usage
//!
//! ### Basic
//!
//! ```rust
//! use rustify::{Client, Endpoint};
//! use rustify_derive::Endpoint;
//!
//! // Defines an API endpoint at /test/path that takes no inputs and returns an
//! // empty response.
//! #[derive(Endpoint)]
//! #[endpoint(path = "test/path")]
//! struct Test {}
//!
//! # tokio_test::block_on(async {
//! let endpoint = Test {};
//! let client = Client::default("http://api.com"); // Configures base address of http://api.com
//! let result = endpoint.exec(&client).await; // Sends GET request to http://api.com/test/path
//!
//! // assert!(result.is_ok());
//! # });
//! ```
//!
//!
//! ### Request Body
//!
//! ```rust
//! use derive_builder::Builder;
//! use rustify::{Client, Endpoint};
//! use rustify_derive::Endpoint;
//!
//! // Defines an API endpoint at /test/path/{name} that takes one input for
//! // creating the url and two inputs for building the request body. The content
//! // type of the request body defaults to JSON, however, it can be modified by
//! // passing the `request_type` parameter to the endpoint configuration.
//! //
//! // Note: The `#[endpoint(body)]` attribute tags are technically optional in the
//! // below example. If no `body` attribute is found anywhere then rustify defaults
//! // to serializing all "untagged" fields as part of the body. Fields can be opted
//! // out of this behavior by tagging them with #[endpoint(skip)].
//! #[derive(Builder, Endpoint)]
//! #[endpoint(path = "test/path/{self.name}", method = "POST", builder = "true")]
//! #[builder(setter(into))] // Improves the building process
//! struct Test {
//!     #[endpoint(skip)] // This field shouldn't be serialized anywhere
//!     pub name: String, // Used to create a dynamic URL
//!     #[endpoint(body)] // Instructs rustify to serialize this field as part of the body
//!     pub age: i32,
//!     #[endpoint(body)]
//!     pub role: String,
//! }
//!
//! // Setting `builder` to true creates a `builder()` method on our struct that
//! // returns the TestBuilder type created by `derive_builder`.
//! # tokio_test::block_on(async {
//! let endpoint = Test::builder()
//!         .name("jmgilman")
//!         .age(42)
//!         .role("CEO")
//!         .build()
//!         .unwrap();
//! let client = Client::default("http://api.com");
//! let result = endpoint.exec(&client).await; // Sends POST request to http://api.com/test/path/jmgilman
//!
//! // assert!(result.is_ok());
//! # });
//! ```
//!
//! ### Query Parameters
//!
//! ```rust
//! use derive_builder::Builder;
//! use rustify::{Client, Endpoint};
//! use rustify_derive::Endpoint;
//!
//! // Defines a similar API endpoint as in the previous example but adds an
//! // optional query parameter to the request. Additionally, this example opts to
//! // not specify the `#[endpoint(body)]` attributes to make use of the default
//! // behavior covered in the previous example.
//! #[derive(Default, Builder, Endpoint)]
//! #[endpoint(path = "test/path/{self.name}", method = "POST", builder = "true")]
//! #[builder(setter(into, strip_option), default)] // Improves building process
//! struct Test {
//!     #[endpoint(skip)]
//!     pub name: String,
//!     #[endpoint(query)]
//!     pub scope: Option<String>, // Note: serialization is skipped when this field is None
//!     pub age: i32, // Serialized into the request body
//!     pub role: String, // Serialized into the request body
//! }
//!
//! # tokio_test::block_on(async {
//! let endpoint = Test::builder()
//!         .name("jmgilman")
//!         .scope("global")
//!         .age(42)
//!         .role("CEO")
//!         .build()
//!         .unwrap();
//! let client = Client::default("http://api.com");
//! let result = endpoint.exec(&client).await; // Sends POST request to http://api.com/test/path/jmgilman?scope=global
//!
//! // assert!(result.is_ok());
//! # });
//! ```
//!
//! ### Responses
//!
//! ```should_panic
//! use rustify::{Client, Endpoint};
//! use rustify_derive::Endpoint;
//! use serde::Deserialize;
//!
//! // Defines an API endpoint at /test/path that takes a single byte array which
//! // will be used as the request body (no serialization occurs). The endpoint
//! // returns a `TestResponse` which contains the result of the operation.
//! #[derive(Endpoint)]
//! #[endpoint(path = "test/path", response = "TestResponse")]
//! struct Test {
//!     #[endpoint(raw)] // Indicates this field contains the raw request body
//!     pub file: Vec<u8>
//! }
//!
//! #[derive(Deserialize)]
//! struct TestResponse {
//!     pub success: bool,
//! }
//!
//! # tokio_test::block_on(async {
//! let endpoint = Test {
//!     file: b"contents".to_vec(),   
//! };
//! let client = Client::default("http://api.com");
//! let result = endpoint.exec(&client).await;
//!
//! // assert!(result.is_ok());
//!
//! let response = result.unwrap().parse().unwrap(); // Returns the parsed `TestResponse`
//! // dbg!(response.success);
//! # });
//! ```
//!
//! ## Examples
//!
//! You can find example usage in the [examples](examples) directory. They can
//! be run with cargo:
//!
//! ```ignore
//! cargo run --package rustify --example reqres1
//! cargo run --package rustify --example reqres2
//! ```
//!
//! The [vaultrs](https://github.com/jmgilman/vaultrs) crate is built upon rustify
//! and serves as as good reference.
//!
//! ## Features
//! The following features are available for this crate:
//!
//! * `blocking`: Enables the blocking variants of `Client`s as well as the blocking
//!    `exec()` functions in `Endpoint`s.
//!
//! ## Error Handling
//!
//! All errors generated by this crate are wrapped in the `ClientError` enum
//! provided by the crate.
//!
//! ## Testing
//!
//! See the the [tests](tests) directory for tests. Run tests with `cargo test`.
//!
//! ## Contributing
//!
//! Check out the [issues][1] for items needing attention or submit your own and
//! then:
//!
//! 1. Fork it (https://github.com/jmgilman/rustify/fork)
//! 2. Create your feature branch (git checkout -b feature/fooBar)
//! 3. Commit your changes (git commit -am 'Add some fooBar')
//! 4. Push to the branch (git push origin feature/fooBar)
//! 5. Create a new Pull Request
//!
//! [1]: https://github.com/jmgilman/rustify/issues

#[macro_use]
extern crate tracing;

#[cfg(feature = "blocking")]
pub mod blocking;
pub mod client;
pub mod clients;
pub mod endpoint;
pub mod enums;
pub mod errors;
pub mod http;

#[doc(hidden)]
#[path = "private/mod.rs"]
pub mod __private;

pub use crate::{
    clients::reqwest::Client,
    endpoint::{Endpoint, MiddleWare, Wrapper},
};
