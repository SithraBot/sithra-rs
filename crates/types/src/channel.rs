use std::{fmt::Display, time::Duration};

use serde::{Deserialize, Serialize};
use sithra_server::{
    extract::context::{Clientful, Context},
    server::PostError,
};
use sithra_transport::channel::Channel;
use thiserror::Error;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SetMute {
    pub channel:  Channel,
    pub duration: Duration,
}

pub mod command {
    use sithra_server::{traits::TypedRequest, typed};

    use super::SetMute;
    use crate::{into_request, into_response};
    typed!("/command/channel.mute" => impl SetMute);

    impl TypedRequest for SetMute {
        type Response = ();
    }

    into_response!("/command/channel.mute", SetMute);
    into_request!("/command/channel.mute", SetMute);
}

pub trait ContextExt {
    /// # Errors
    /// - Context not included channel
    /// - Post error
    fn set_mute_member(
        &self,
        user: impl Display + Send + Sync,
        duration: Duration,
    ) -> impl Future<Output = Result<(), SetMuteError>>;

    /// # Errors
    /// - Context not included channel
    /// - Post error
    fn set_mute(
        &self,
        channel: Channel,
        duration: Duration,
    ) -> impl Future<Output = Result<(), SetMuteError>>;
}

impl<T, S> ContextExt for Context<T, S>
where
    S: Clientful + Send + Sync,
    T: for<'de> Deserialize<'de> + Send + Sync,
{
    async fn set_mute_member(
        &self,
        user: impl Display + Send + Sync,
        duration: Duration,
    ) -> Result<(), SetMuteError> {
        let mut channel = self
            .request
            .channel()
            .ok_or(SetMuteError::ContextNotIncludedChannel)?;
        channel.id = user.to_string();
        let set_mute = SetMute { channel, duration };
        self.post(set_mute).await?;
        Ok(())
    }

    async fn set_mute(
        &self,
        channel: Channel,
        duration: Duration,
    ) -> Result<(), SetMuteError> {
        let set_mute = SetMute { channel, duration };
        self.post(set_mute).await?;
        Ok(())
    }
}

#[derive(Debug, Error)]
#[allow(clippy::large_enum_variant)]
pub enum SetMuteError {
    #[error("Post error: {0}")]
    Post(#[from] PostError),
    #[error("Context not included channel")]
    ContextNotIncludedChannel,
}
