//! Utility functions for transport operations
//!
//! Provides helper functions for creating framed transports and chunking data.

use std::io::Write;

use bytes::{BufMut, Bytes, BytesMut};
use serde::Serialize;
use tokio::process::Child;
use tokio_util::codec::Framed;

use crate::{
    datapack::{DataPackCodec, RawDataPackCodec},
    peer::Peer,
};

pub type FramedPeer = Framed<Peer, DataPackCodec>;

#[must_use]
pub fn framed(peer: Peer) -> Framed<Peer, DataPackCodec> {
    Framed::new(peer, DataPackCodec::new())
}

/// Connects to a child process and returns a framed transport.
///
/// # Errors
/// Returns an error if the child process fails to start or if the framed
/// transport fails to be created.
#[allow(clippy::result_large_err)]
/// Creates a framed transport for structured data communication with a child
/// process
///
/// # Arguments
/// * `child` - The child process to connect to
///
/// # Returns
/// Framed transport using `DataPackCodec` for structured messages
pub fn connect(child: Child) -> Result<Framed<Peer, DataPackCodec>, Child> {
    let peer = Peer::from_child(child)?;
    let codec = DataPackCodec::new();
    Ok(Framed::new(peer, codec))
}

/// Connects to a child process and returns a framed transport using raw data
/// packing.
///
/// # Errors
/// Returns an error if the child process fails to start or if the framed
/// transport fails to be created.
#[allow(clippy::result_large_err)]
/// Creates a framed transport for raw data communication with a child process
///
/// # Arguments
/// * `child` - The child process to connect to
///
/// # Returns
/// Framed transport using `RawDataPackCodec` for raw byte messages
pub fn raw_connect(child: Child) -> Result<Framed<Peer, RawDataPackCodec>, Child> {
    let peer = Peer::from_child(child)?;
    let codec = RawDataPackCodec::new();
    Ok(Framed::new(peer, codec))
}

/// Creates a framed transport using standard input and output.
#[must_use]
/// Creates a framed transport using standard input/output for structured data
///
/// # Returns
/// Framed transport using `DataPackCodec` with stdin/stdout
pub fn stdio() -> Framed<Peer, DataPackCodec> {
    let peer = Peer::new();
    let codec = DataPackCodec::new();
    Framed::new(peer, codec)
}

/// Creates a framed transport using standard input and output.
#[must_use]
/// Creates a framed transport using standard input/output for raw data
///
/// # Returns
/// Framed transport using `RawDataPackCodec` with stdin/stdout
pub fn raw_stdio() -> Framed<Peer, RawDataPackCodec> {
    let peer = Peer::new();
    let codec = RawDataPackCodec::new();
    Framed::new(peer, codec)
}

/// Serializes a value into a `Bytes` buffer
///
/// # Arguments
/// * `val` - The value to serialize
///
/// # Errors
/// Returns an error if serialization fails
///
/// # Returns
/// Result containing the serialized `Bytes` buffer, or an error if
/// serialization fails
pub fn to_bytes<S: Serialize + ?Sized>(val: &S) -> Result<Bytes, crate::EncodeError> {
    let mut writer = BytesMut::new().writer();
    rmp_serde::encode::write_named(&mut writer, val)?;
    Ok(writer.into_inner().freeze())
}

/// Serializes a value into a `Bytes` buffer and writes it to a writer
///
/// # Arguments
/// * `wr` - The writer to write the serialized value to
/// * `val` - The value to serialize
///
/// # Errors
/// Returns an error if serialization fails
///
/// # Returns
/// Result containing the serialized `Bytes` buffer, or an error if
/// serialization fails
pub fn write_bytes<S: Serialize + ?Sized, W: Write + ?Sized>(
    wr: &mut W,
    val: &S,
) -> Result<(), crate::EncodeError> {
    rmp_serde::encode::write_named(wr, val)
}

#[cfg(test)]
mod tests {}
