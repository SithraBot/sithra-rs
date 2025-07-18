use std::ops::{Deref, DerefMut};

use serde::de::DeserializeOwned;
use sithra_transport::{datapack::RequestDataPack, Value, ValueError};
use triomphe::Arc;

use crate::{extract::FromRequest, response};

#[derive(Debug, Default, Clone, Copy)]
pub struct Payload<T>(pub T);

impl<T> Payload<T> {
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> From<T> for Payload<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}

impl<T: DeserializeOwned> Payload<T> {
    /// # Errors
    /// Returns an error if the value cannot be deserialized.
    pub fn from_value(value: &Value) -> Result<Self, ValueError> {
        Ok(Self(sithra_transport::from_value(value.clone())?))
    }
}

impl<T, S> FromRequest<S> for Payload<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = response::Error<ValueError>;

    async fn from_request(req: Arc<RequestDataPack>, _: &S) -> Result<Self, Self::Rejection> {
        Ok(Self::from_value(&req.payload)?)
    }
}

impl<T> Deref for Payload<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Payload<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
