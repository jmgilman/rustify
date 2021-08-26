//! Contains implementations of [Client][crate::blocking::client::Client] which
//! use varying blocking HTTP clients.
#[cfg(feature = "reqwest")]
pub mod reqwest;
