use std::time::Duration;

use serde::{Deserialize, Serialize};
use sithra_transport::channel::Channel;

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
