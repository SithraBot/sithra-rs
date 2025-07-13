use log::Level;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Log {
    pub level:   Level,
    pub message: String,
    pub target:  String,
}

impl Log {
    #[must_use]
    pub fn new(level: impl Into<Level>, message: String, target: String) -> Self {
        Self {
            level: level.into(),
            message,
            target,
        }
    }
}

pub mod command {
    use sithra_server::typed;

    use super::Log;
    use crate::{into_request, into_response};

    typed!("/log.create" => impl Log);
    into_response!("/log.create", Log);
    into_request!("/log.create", Log);
}
