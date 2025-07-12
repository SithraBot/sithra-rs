use std::{
    convert::Infallible,
    ops::{Deref, DerefMut},
};

use crate::extract::FromRequest;

pub struct BotId(pub Option<String>);

impl Deref for BotId {
    type Target = Option<String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for BotId {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<BotId> for Option<String> {
    fn from(value: BotId) -> Self {
        value.0
    }
}

impl From<Option<String>> for BotId {
    fn from(value: Option<String>) -> Self {
        Self(value)
    }
}

impl<S: Send + Sync> FromRequest<S> for BotId {
    type Rejection = Infallible;

    async fn from_request(
        req: triomphe::Arc<sithra_transport::datapack::RequestDataPack>,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        Ok(Self(req.bot_id.clone()))
    }
}
