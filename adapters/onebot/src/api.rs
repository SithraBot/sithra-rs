pub mod response {
    use serde::{Deserialize, Serialize};
    use sithra_kit::{
        transport::datapack::DataPack,
        types::{message::Message, smallvec::SmallVec},
    };
    use ulid::Ulid;

    use crate::util::de_str_from_num;

    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub struct ApiResponse {
        retcode:  i32,
        #[serde(default)]
        status:   String,
        echo:     Ulid,
        pub data: Option<ApiResponseKind>,
    }

    impl ApiResponse {
        #[must_use]
        pub fn into_rep(self, bot_id: &str) -> DataPack {
            let Self {
                retcode,
                status,
                echo,
                data,
            } = self;
            if retcode >= 400 {
                return DataPack::builder().correlate(echo).bot_id(bot_id).build_with_error(
                    format!("Call OneBot API Error, RETCODE: {retcode}, STATUS: {status}"),
                );
            }
            let Some(data) = data else {
                return DataPack::builder().correlate(echo).bot_id(bot_id).build_with_payload(&());
            };
            match data {
                ApiResponseKind::SendMessage(send_msg) => {
                    let payload: Message = Message {
                        id:      send_msg.message_id,
                        content: SmallVec::new(),
                    };
                    DataPack::builder().correlate(echo).build_with_payload(&payload)
                }
                ApiResponseKind::GetStatus(status) => {
                    DataPack::builder().correlate(echo).build_with_payload(&status)
                }
            }
        }
    }

    #[derive(Debug, Clone, Deserialize, Serialize)]
    #[serde(untagged)]
    pub enum ApiResponseKind {
        SendMessage(SendMessageResponse),
        GetStatus(Status),
    }

    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub struct Status {
        pub good: bool,
    }

    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub struct SendMessageResponse {
        #[serde(deserialize_with = "de_str_from_num")]
        message_id: String,
    }
}

pub mod request {
    use std::fmt::Display;

    use serde::{Deserialize, Serialize};
    use ulid::Ulid;

    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub struct ApiCall<T> {
        pub action: String,
        pub params: T,
        pub echo:   Ulid,
    }

    impl<T> ApiCall<T> {
        pub fn new(action: impl Display, params: T, echo: Ulid) -> Self {
            Self {
                action: action.to_string(),
                params,
                echo,
            }
        }
    }
}
