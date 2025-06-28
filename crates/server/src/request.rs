use serde::Deserialize;
use sithra_transport::{channel::Channel, datapack::RequestDataPack};
use triomphe::Arc;
use ulid::Ulid;

#[derive(Clone, Debug)]
pub struct Request {
    pub data: Arc<RequestDataPack>,
}

impl From<RequestDataPack> for Request {
    fn from(value: RequestDataPack) -> Self {
        Self::new(value)
    }
}

impl From<Arc<RequestDataPack>> for Request {
    fn from(value: Arc<RequestDataPack>) -> Self {
        Self { data: value }
    }
}

impl Request {
    #[must_use]
    pub fn correlation(&self) -> Ulid {
        self.data.correlation()
    }

    #[must_use]
    pub fn into_raw(self) -> Arc<RequestDataPack> {
        self.data
    }

    #[must_use]
    pub const fn from_raw(data: Arc<RequestDataPack>) -> Self {
        Self { data }
    }

    #[must_use]
    pub fn new(data: RequestDataPack) -> Self {
        Self {
            data: Arc::new(data),
        }
    }

    /// # Errors
    /// Returns an error if the payload cannot be deserialized.
    pub fn payload<T: for<'de> Deserialize<'de>>(&self) -> Result<T, rmpv::ext::Error> {
        rmpv::ext::from_value(self.data.payload.clone())
    }

    #[must_use]
    pub fn channel(&self) -> Option<Channel> {
        self.data.channel.clone()
    }
}
