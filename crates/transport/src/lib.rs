//! Transport layer implementation for sithra-rs.
//!
//! This crate provides core networking abstractions including:
//! - [`channel`]: Channel management for message passing
//! - [`datapack`]: Structured data packet serialization
//! - [`peer`]: Peer connection management
//! - [`util`]: Shared utilities
//!
//! # Features
//! - Async I/O using tokio

#![allow(clippy::cast_possible_truncation)]

pub mod channel;
pub mod datapack;
pub mod peer;
pub mod util;

pub use rmp_serde::{decode::Error as DecodeError, encode::Error as EncodeError};
pub use serde_json::{Error as ValueError, Value, from_value, to_value};
