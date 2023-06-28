//! Contains implementations of [Client][crate::client::Client] which use
//! varying HTTP clients.
#[cfg(feature = "reqwest")]
pub mod reqwest;
#[cfg(all(feature = "reqwest", feature = "reqwest-middleware"))]
pub mod reqwest_middleware;
