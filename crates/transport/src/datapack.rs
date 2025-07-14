use std::fmt::Display;

use bytes::{Buf, BufMut, Bytes, BytesMut};
use either::Either;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio_util::codec::{Decoder, Encoder};
use ulid::Ulid;

use crate::{channel::Channel, util::get_chunk};

/// A raw data packet containing a length-prefixed byte buffer.
///
/// Used for low-level serialization/deserialization of data packets.
/// The `data_len` field stores the length of the `data` field in bytes.
#[derive(Clone)]
pub struct RawDataPack {
    data_len: u32,
    pub data: Bytes,
}

impl RawDataPack {
    /// Creates a new `RawDataPack` from the given byte buffer.
    ///
    /// The `data_len` field is automatically calculated from the buffer length.
    const fn new(data: Bytes) -> Self {
        Self {
            data_len: data.len() as u32,
            data,
        }
    }
}

/// A codec for encoding/decoding `RawDataPack` instances.
///
/// Maintains internal buffers for partial reads/writes and tracks
/// the current packet length during decoding.
pub struct RawDataPackCodec {
    data_len:  Option<u32>,
    de_buffer: BytesMut,
    en_buffer: BytesMut,
}

impl RawDataPackCodec {
    /// Creates a new `RawDataPackCodec` with empty buffers.
    #[must_use]
    pub fn new() -> Self {
        Self {
            data_len:  None,
            de_buffer: BytesMut::new(),
            en_buffer: BytesMut::new(),
        }
    }
}

impl Default for RawDataPackCodec {
    /// Creates a default `RawDataPackCodec` instance.
    fn default() -> Self {
        Self::new()
    }
}

impl Encoder<RawDataPack> for RawDataPackCodec {
    type Error = std::io::Error;

    /// Encodes a `RawDataPack` into the destination buffer.
    ///
    /// Writes the length prefix followed by the data payload.
    fn encode(&mut self, item: RawDataPack, dst: &mut BytesMut) -> Result<(), Self::Error> {
        self.en_buffer.put_u32(item.data_len);
        self.en_buffer.put(item.data);
        while let Some(bytes) = get_chunk(&mut self.en_buffer) {
            dst.put(bytes);
        }
        Ok(())
    }
}

impl Decoder for RawDataPackCodec {
    type Error = std::io::Error;
    type Item = RawDataPack;

    /// Decodes a `RawDataPack` from the source buffer.
    ///
    /// Reads the length prefix first, then the data payload once enough bytes
    /// are available.
    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        self.de_buffer.put(src.split());
        if self.de_buffer.len() < 4 && self.data_len.is_none() {
            return Ok(None);
        } else if self.data_len.is_none() {
            self.data_len = Some(self.de_buffer.get_u32());
        }
        let Some(data_len) = self.data_len else {
            return Ok(None);
        };
        if self.de_buffer.len() < (data_len as usize) {
            return Ok(None);
        }
        let data = self.de_buffer.split_to(data_len as usize);
        self.data_len = None;
        Ok(Some(Self::Item {
            data_len,
            data: data.into(),
        }))
    }
}

/// A structured data packet for communication between peers.
///
/// Contains optional metadata (`path`, `channel`), a correlation ID,
/// and a `result` field that can be either a payload or an error.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DataPack {
    pub bot_id:      Option<String>,
    pub path:        Option<String>,
    pub correlation: Ulid,
    pub channel:     Option<Channel>,
    #[serde(flatten)]
    pub result:      DataResult,
}

impl Default for DataPack {
    fn default() -> Self {
        Self {
            bot_id:      None,
            path:        None,
            correlation: Ulid::new(),
            channel:     None,
            result:      DataResult::Payload(crate::Value::Null),
        }
    }
}

impl From<RequestDataPack> for DataPack {
    fn from(value: RequestDataPack) -> Self {
        let RequestDataPack {
            bot_id,
            path,
            correlation,
            channel,
            payload,
        } = value;
        Self {
            bot_id,
            path: Some(path),
            correlation,
            channel,
            result: DataResult::Payload(payload),
        }
    }
}

/// A request-specific data packet with required path and payload.
///
/// Used for initiating requests between peers, with optional channel
/// metadata and a correlation ID for tracking.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RequestDataPack {
    pub bot_id:  Option<String>,
    pub path:    String,
    correlation: Ulid,
    pub channel: Option<Channel>,
    pub payload: crate::Value,
}

impl Default for RequestDataPack {
    /// Creates a default `RequestDataPack` with empty fields.
    fn default() -> Self {
        Self {
            bot_id:      None,
            path:        String::new(),
            correlation: Ulid::new(),
            channel:     None,
            payload:     crate::Value::Null,
        }
    }
}

impl RequestDataPack {
    #[must_use]
    pub fn bot_id(mut self, bot_id: impl Display) -> Self {
        self.bot_id = Some(bot_id.to_string());
        self
    }

    #[must_use]
    pub fn path(mut self, path: impl Display) -> Self {
        self.path = path.to_string();
        self
    }

    #[must_use]
    pub fn channel(mut self, channel: Channel) -> Self {
        self.channel = Some(channel);
        self
    }

    #[must_use]
    pub fn channel_opt(mut self, channel: Option<Channel>) -> Self {
        if let Some(channel) = channel {
            self.channel = Some(channel);
        }
        self
    }

    #[must_use]
    pub fn payload_value(mut self, payload: impl Into<crate::Value>) -> Self {
        self.payload = payload.into();
        self
    }

    #[must_use]
    pub fn payload<S: Serialize>(mut self, payload: S) -> Self {
        self.payload = crate::to_value(payload).unwrap_or(crate::Value::Null);
        self
    }

    #[must_use]
    /// Returns the correlation ID of the `DataPack`.
    pub const fn correlation(&self) -> Ulid {
        self.correlation
    }

    #[must_use]
    pub fn link(mut self, other: &Self) -> Self {
        if self.bot_id.is_none() {
            self.bot_id.clone_from(&other.bot_id);
        }
        if self.channel.is_none() {
            self.channel.clone_from(&other.channel);
        }
        self.correlation = other.correlation();
        self
    }
}

/// Represents the result of a data operation, either a payload or an error.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum DataResult {
    /// Successful operation with a payload value.
    #[serde(rename = "payload")]
    Payload(crate::Value),
    /// Failed operation with an error message.
    #[serde(rename = "error")]
    Error(String),
}

/// Converts a `DataResult` into a standard `Result`.
///
/// - `Payload(v)` becomes `Ok(v)`
/// - `Error(e)` becomes `Err(e)`
impl From<DataResult> for Result<crate::Value, String> {
    fn from(value: DataResult) -> Self {
        match value {
            DataResult::Payload(v) => Ok(v),
            DataResult::Error(e) => Err(e),
        }
    }
}

/// Converts a standard `Result` into a `DataResult`.
///
/// - `Ok(payload)` becomes `Payload(payload.into())`
/// - `Err(error)` becomes `Error(error.to_string())`
impl<P, E> From<Result<P, E>> for DataResult
where
    P: Into<crate::Value>,
    E: Display,
{
    fn from(value: Result<P, E>) -> Self {
        match value {
            Ok(payload) => Self::Payload(payload.into()),
            Err(error) => Self::Error(error.to_string()),
        }
    }
}

/// A builder for constructing `DataPack` instances with optional fields.
///
/// Provides a fluent interface for setting fields like `path`, `correlation`,
/// `channel`, and `result` before building the final `DataPack`.
pub struct DataPackBuilder {
    pub bot_id:      Option<String>,
    pub path:        Option<String>,
    pub correlation: Option<Ulid>,
    pub channel:     Option<Channel>,
    pub result:      Option<DataResult>,
}

impl Default for DataPackBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl DataPackBuilder {
    /// Creates a new `DataPackBuilder` with all fields set to `None`.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            bot_id:      None,
            path:        None,
            correlation: None,
            channel:     None,
            result:      None,
        }
    }

    /// Sets the `bot_id` field for the `DataPack`.
    #[must_use]
    pub fn bot_id(mut self, id: impl Display) -> Self {
        self.bot_id = Some(id.to_string());
        self
    }

    /// Sets the `path` field for the `DataPack`.
    #[must_use]
    pub fn path(mut self, path: impl Display) -> Self {
        self.path = Some(path.to_string());
        self
    }

    #[must_use]
    pub fn correlate(mut self, id: impl Into<Ulid>) -> Self {
        self.correlation = Some(id.into());
        self
    }

    /// Sets the `channel` field for the `DataPack`.
    #[must_use]
    pub fn channel(mut self, id: impl Into<Channel>) -> Self {
        self.channel = Some(id.into());
        self
    }

    /// Sets the `result` field for the `DataPack`.
    #[must_use]
    pub fn result(mut self, result: impl Into<DataResult>) -> Self {
        self.result = Some(result.into());
        self
    }

    /// Builds the `DataPack` with the configured fields.
    ///
    /// Defaults:
    /// - `correlation`: A new `Ulid` if not set.
    /// - `result`: `DataResult::Payload(crate::Value::Null)` if not set.
    #[must_use]
    pub fn build(self) -> DataPack {
        let Self {
            bot_id,
            path,
            correlation,
            channel,
            result,
        } = self;

        let correlation = correlation.unwrap_or_else(Ulid::new);

        let result = result.unwrap_or(DataResult::Payload(crate::Value::Null));

        DataPack {
            bot_id,
            path,
            correlation,
            channel,
            result,
        }
    }

    /// Sets the `result` field to a `Payload` variant.
    #[must_use]
    pub fn payload(mut self, payload: impl Serialize) -> Self {
        let payload = crate::to_value(payload);
        match payload {
            Ok(payload) => {
                self.result = Some(DataResult::Payload(payload));
            }
            Err(err) => {
                return self.error(err);
            }
        }
        self
    }

    /// Sets the `result` field to an `Error` variant.
    #[must_use]
    pub fn error(mut self, error: impl Display) -> Self {
        self.result = Some(DataResult::Error(error.to_string()));
        self
    }

    /// Builds a `DataPack` with a `Payload` result.
    #[must_use]
    pub fn build_with_payload(mut self, payload: impl Serialize) -> DataPack {
        let payload = crate::to_value(payload);
        match payload {
            Ok(payload) => {
                self.result = Some(DataResult::Payload(payload));
            }
            Err(err) => {
                return self.error(err).build();
            }
        }
        self.build()
    }

    /// Builds a `DataPack` with an `Error` result.
    #[must_use]
    pub fn build_with_error(self, error: impl Display) -> DataPack {
        self.error(error).build()
    }
}

impl DataPack {
    #[must_use]
    pub const fn builder() -> DataPackBuilder {
        DataPackBuilder::new()
    }

    #[must_use]
    pub fn link(mut self, other: &Self) -> Self {
        if self.bot_id.is_none() {
            self.bot_id.clone_from(&other.bot_id);
        }
        if self.channel.is_none() {
            self.channel.clone_from(&other.channel);
        }
        self.correlate(other.correlation());
        self
    }

    /// # Errors
    /// Returns an error if deserialization fails.
    pub fn payload<T: for<'de> Deserialize<'de>>(&self) -> Result<T, String> {
        let payload = match &self.result {
            DataResult::Error(err) => return Err(err.clone()),
            DataResult::Payload(payload) => payload.clone(),
        };
        crate::from_value(payload).map_err(|err| format!("{err}"))
    }

    /// Deserialize a `DataPack` from a byte slice.
    ///
    /// # Errors
    /// Returns an error if the byte slice is not a valid `DataPack`.
    /// Deserialize a `DataPack` from a byte slice.
    ///
    /// # Errors
    /// Returns an error if the byte slice is not a valid `DataPack`.
    pub fn deserialize(bytes: &[u8]) -> Result<Self, rmp_serde::decode::Error> {
        rmp_serde::from_slice(bytes)
    }

    /// Serialize a `DataPack` into a byte slice.
    ///
    /// # Errors
    /// Returns an error if the `DataPack` cannot be serialized.
    /// Serialize a `DataPack` into a byte slice.
    ///
    /// # Errors
    /// Returns an error if the `DataPack` cannot be serialized.
    pub fn serialize(&self) -> Result<Bytes, rmp_serde::encode::Error> {
        rmp_serde::to_vec_named(self).map(Bytes::from)
    }

    /// Serialize a `DataPack` into a raw byte slice.
    ///
    /// # Errors
    /// Returns an error if the `DataPack` cannot be serialized.
    /// Serialize a `DataPack` into a raw byte slice.
    ///
    /// # Errors
    /// Returns an error if the `DataPack` cannot be serialized.
    pub fn serialize_to_raw(&self) -> Result<RawDataPack, rmp_serde::encode::Error> {
        rmp_serde::to_vec_named(self).map(|v| RawDataPack::new(Bytes::from(v)))
    }

    #[must_use]
    pub const fn correlation(&self) -> Ulid {
        self.correlation
    }

    #[must_use]
    /// Checks if the `DataPack` represents a request (has a path).
    pub const fn is_request(&self) -> bool {
        self.path.is_some()
    }

    #[must_use]
    pub fn either_request(self) -> Either<Self, RequestDataPack> {
        if self.is_request() {
            Either::Right(self.into_request())
        } else {
            Either::Left(self)
        }
    }

    /// Sets the correlation ID of the `DataPack`.
    pub const fn correlate(&mut self, id: Ulid) {
        self.correlation = id;
    }

    #[must_use]
    pub fn into_request(self) -> RequestDataPack {
        let Self {
            bot_id,
            path,
            correlation,
            channel,
            result,
        } = self;
        let payload: Result<_, _> = result.into();
        RequestDataPack {
            bot_id,
            path: path.unwrap_or_default(),
            correlation,
            channel,
            payload: payload.unwrap_or(crate::Value::Null),
        }
    }
}

/// A codec for encoding/decoding `DataPack` instances.
///
/// Wraps a `RawDataPackCodec` to handle the low-level byte operations
/// while providing higher-level `DataPack` serialization/deserialization.
pub struct DataPackCodec {
    raw: RawDataPackCodec,
}

impl DataPackCodec {
    /// Creates a new `DataPackCodec` with a default `RawDataPackCodec`.
    #[must_use]
    pub fn new() -> Self {
        Self {
            raw: RawDataPackCodec::new(),
        }
    }
}

impl Default for DataPackCodec {
    fn default() -> Self {
        Self::new()
    }
}

impl Decoder for DataPackCodec {
    /// Decodes a `DataPack` from raw bytes.
    type Error = DataPackCodecError;
    type Item = DataPack;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let raw_data = self.raw.decode(src)?;
        let Some(raw_data) = raw_data else {
            return Ok(None);
        };
        Ok(Some(DataPack::deserialize(&raw_data.data)?))
    }
}

impl Encoder<&DataPack> for DataPackCodec {
    /// Encodes a `DataPack` into raw bytes.
    type Error = DataPackCodecError;

    fn encode(&mut self, item: &DataPack, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let raw_data = item.serialize_to_raw()?;
        self.raw.encode(raw_data, dst)?;
        Ok(())
    }
}

impl Encoder<DataPack> for DataPackCodec {
    /// Encodes a `DataPack` into raw bytes.
    type Error = DataPackCodecError;

    fn encode(&mut self, item: DataPack, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let raw_data = item.serialize_to_raw()?;
        self.raw.encode(raw_data, dst)?;
        Ok(())
    }
}

impl Encoder<RawDataPack> for DataPackCodec {
    /// Encodes a `RawDataPack` directly into the destination buffer.
    type Error = DataPackCodecError;

    fn encode(&mut self, item: RawDataPack, dst: &mut BytesMut) -> Result<(), Self::Error> {
        self.raw.encode(item, dst)?;
        Ok(())
    }
}

/// Error type for `DataPackCodec` operations.
#[derive(Debug, Error)]
pub enum DataPackCodecError {
    /// Wraps I/O errors during encoding/decoding.
    #[error("IO error in DataPack codec: {0}")]
    IO(#[from] std::io::Error),
    /// Wraps serialization errors when converting `DataPack` to bytes.
    #[error("DataPack serialization error: {0}")]
    Serialize(#[from] rmp_serde::encode::Error),
    /// Wraps deserialization errors when converting bytes to `DataPack`.
    #[error("DataPack deserialization error: {0}")]
    Deserialize(#[from] rmp_serde::decode::Error),
}
