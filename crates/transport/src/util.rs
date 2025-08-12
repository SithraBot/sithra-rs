//! Utility functions for transport operations
//!
//! Provides helper functions for creating framed transports and chunking data.

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

/// Splits data into chunks of maximum 1024 bytes
///
/// # Arguments
/// * `src` - The source buffer to chunk
///
/// # Returns
/// Some(BytesMut) containing up to 1024 bytes, or None if empty
pub fn get_chunk(src: &mut BytesMut) -> Option<BytesMut> {
    if src.is_empty() {
        None
    } else if src.len() < 1024 {
        Some(src.split_to(src.len()))
    } else {
        Some(src.split_to(1024))
    }
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
/// Result containing the serialized `Bytes` buffer, or an error if serialization fails
pub fn to_bytes<S: Serialize + ?Sized>(val: &S) -> Result<Bytes, crate::EncodeError> {
    let mut writer = BytesMut::new().writer();
    rmp_serde::encode::write_named(&mut writer, val)?;
    Ok(writer.into_inner().freeze())
}

#[cfg(test)]
mod tests {
    use bytes::Bytes;

    use super::*;

    #[test]
    fn test_get_chunk() {
        // Test with an empty buffer
        let mut src = BytesMut::new();
        let chunk = get_chunk(&mut src);
        assert_eq!(chunk, None);
        assert_eq!(src, BytesMut::new());

        // Test with a buffer smaller than the chunk size
        let mut src = BytesMut::from("Hello, world!");
        let chunk = get_chunk(&mut src);
        assert_eq!(chunk, Some(BytesMut::from("Hello, world!")));
        assert_eq!(src, BytesMut::new());

        // Test with a buffer larger than the chunk size
        let mut src = BytesMut::from(Bytes::from(vec![10u8; 2048]));
        let chunk = get_chunk(&mut src);
        assert_eq!(chunk, Some(BytesMut::from(Bytes::from(vec![10u8; 1024]))));
        assert_eq!(src, BytesMut::from(Bytes::from(vec![10u8; 1024])));

        // Test with a buffer exactly the chunk size
        let mut src = BytesMut::from(Bytes::from(vec![10u8; 1024]));
        let chunk = get_chunk(&mut src);
        assert_eq!(chunk, Some(BytesMut::from(Bytes::from(vec![10u8; 1024]))));
        assert_eq!(src, BytesMut::new());
    }
}
