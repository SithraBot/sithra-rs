use serde::Deserialize;
use sithra_transport::{channel::Channel, datapack::RequestDataPack, ValueError};
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
    pub fn raw(&self) -> &RequestDataPack {
        &self.data
    }

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

    #[must_use]
    pub fn bot_id(&self) -> Option<String> {
        self.data.bot_id.clone()
    }

    #[must_use]
    pub fn bot_id_ref(&self) -> Option<&str> {
        self.data.bot_id.as_deref()
    }

    /// # Errors
    /// Returns an error if the payload cannot be deserialized.
    pub fn payload<T: for<'de> Deserialize<'de>>(&self) -> Result<T, ValueError> {
        sithra_transport::from_value(self.data.payload.clone())
    }

    #[must_use]
    pub fn channel(&self) -> Option<Channel> {
        self.data.channel.clone()
    }
}
