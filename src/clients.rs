//! Contains implementations of [Client][crate::client::Client] which use
//! varying HTTP clients.
#[cfg(feature = "reqwest")]
pub mod reqwest;
