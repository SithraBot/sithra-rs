use std::fmt::Display;

use serde::{Deserialize, Serialize};
use sithra_server::{
    extract::context::{Clientful, Context},
    server::PostError,
};
use sithra_transport::{channel::Channel, datapack::RequestDataPack, Value, ValueError};
use smallvec::SmallVec;
use typeshare::typeshare;

pub const NIL: sithra_transport::Value = sithra_transport::Value::Null;

pub type Segments<Seg> = SmallVec<[Seg; 1]>;

#[typeshare]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Message<Seg = Segment> {
    pub id:      String,
    #[typeshare(serialized_as = "Vec<Seg>")]
    pub content: SmallVec<[Seg; 1]>,
}

#[typeshare]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Segment {
    #[serde(rename = "type")]
    pub ty:   String,
    #[typeshare(serialized_as = "any")]
    pub data: Value,
}

impl Segment {
    pub fn text<T: Display>(content: T) -> Self {
        Self {
            ty:   "text".to_owned(),
            data: content.to_string().into(),
        }
    }

    pub fn image<T: Display>(url: T) -> Self {
        Self {
            ty:   "image".to_owned(),
            data: url.to_string().into(),
        }
    }

    pub fn img<T: Display>(url: T) -> Self {
        Self::image(url)
    }

    pub fn at<T: Display>(target: T) -> Self {
        Self {
            ty:   "at".to_owned(),
            data: target.to_string().into(),
        }
    }

    /// # Errors
    pub fn custom<T: Display, V: Serialize>(ty: T, data: V) -> Result<Self, ValueError> {
        Ok(Self {
            ty:   ty.to_string(),
            data: sithra_transport::to_value(data)?,
        })
    }
}

impl From<String> for Segment {
    fn from(value: String) -> Self {
        Self::text(value)
    }
}

impl From<&str> for Segment {
    fn from(value: &str) -> Self {
        Self::text(value)
    }
}

#[macro_export]
macro_rules! msg {
    ($seg:ident[$($segment:ident$(: $value:expr)?),*$(,)?]) => {
        [
            $(
                $seg::$segment($($value)?),
            )*
        ].into_iter().collect::<$crate::smallvec::SmallVec<[$seg; 1]>>()
    };
}

#[macro_export]
macro_rules! smsg {
    ($seg:ident[$($segment:ident$(: $value:expr)?),*$(,)?]) => {
        $crate::message::SendMessage::from($crate::msg!($seg[$($segment$(: $value)?),*]))
    };
    ($seg:expr) => {
       $crate::message::SendMessage::from($seg)
    }
}

#[typeshare]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SendMessage<Seg = Segment> {
    #[typeshare(serialized_as = "Vec<Seg>")]
    pub content: SmallVec<[Seg; 1]>,
}

impl SendMessage {
    #[must_use]
    pub fn new<Seg: Into<Segment>>(content: SmallVec<[Seg; 1]>) -> Self {
        Self {
            content: content.into_iter().map(Into::into).collect(),
        }
    }
}

// impl<Seg: Into<Segment>> From<SmallVec<[Seg; 1]>> for SendMessage {
//     fn from(content: SmallVec<[Seg; 1]>) -> Self {
//         Self {
//             content: content.into_iter().map(Into::into).collect(),
//         }
//     }
// }

impl<Seg: TryInto<Segment>> From<SmallVec<[Seg; 1]>> for SendMessage {
    fn from(content: SmallVec<[Seg; 1]>) -> Self {
        Self {
            content: content.into_iter().filter_map(|seg| seg.try_into().ok()).collect(),
        }
    }
}

impl From<String> for SendMessage {
    fn from(content: String) -> Self {
        Self {
            content: SmallVec::from([content.into()]),
        }
    }
}

impl From<&str> for SendMessage {
    fn from(content: &str) -> Self {
        Self {
            content: SmallVec::from([content.into()]),
        }
    }
}

impl<D1: Display, D2: Display> From<Result<D1, D2>> for SendMessage {
    fn from(value: Result<D1, D2>) -> Self {
        let content = match value {
            Ok(content) => content.to_string(),
            Err(err) => err.to_string(),
        };
        Self {
            content: SmallVec::from([content.into()]),
        }
    }
}

pub trait ContextExt {
    fn reply(
        &self,
        msg: impl Into<SendMessage> + Send + Sync,
    ) -> impl Future<Output = Result<Message, PostError>> + Send + Sync;
}

impl<S, Seg> ContextExt for Context<Message<Seg>, S>
where
    S: Clientful + Send + Sync,
    Seg: for<'de> Deserialize<'de> + Send + Sync,
{
    async fn reply(&self, msg: impl Into<SendMessage> + Send + Sync) -> Result<Message, PostError> {
        let datapack = self
            .client()
            .post(
                RequestDataPack::default()
                    .path("/command/message.create")
                    .channel_opt(self.request.channel())
                    .payload(msg.into()),
            )?
            .await?;
        let msg = datapack.payload::<Message>()?;
        Ok(msg)
    }
}

pub trait ClientfulExt {
    fn send_message(
        &self,
        channel: impl Into<Channel> + Send + Sync,
        msg: impl Into<SendMessage> + Send + Sync,
    ) -> impl Future<Output = Result<Message, PostError>> + Send + Sync;
}

impl<C> ClientfulExt for C
where
    C: Clientful + Send + Sync,
{
    async fn send_message(
        &self,
        channel: impl Into<Channel> + Send + Sync,
        msg: impl Into<SendMessage> + Send + Sync,
    ) -> Result<Message, PostError> {
        let datapack = self
            .client()
            .post(
                RequestDataPack::default()
                    .path("/command/message.create")
                    .channel(channel.into())
                    .payload(msg.into()),
            )?
            .await?;
        let msg = datapack.payload::<Message>()?;
        Ok(msg)
    }
}

pub mod event {
    use sithra_server::typed;

    use super::Message;
    typed!("/event/message.created" => impl Message);
}

pub mod command {
    use sithra_server::{traits::TypedRequest, typed};

    use super::SendMessage;
    use crate::{into_request, into_response, message::Message};
    typed!("/command/message.create" => impl SendMessage);

    impl TypedRequest for SendMessage {
        type Response = Message;
    }

    into_response!("/command/message.create", SendMessage);
    into_request!("/command/message.create", SendMessage);
}

pub mod common {
    use std::fmt::Display;

    use de::Error as _;
    use serde::{Deserialize, Serialize, de};
    use sithra_transport::ValueError;

    use crate::message::Segment;

    #[derive(Debug, Clone)]
    pub enum CommonSegment {
        Text(String),
        Image(String),
        At(String),
        Unknown(Segment),
    }

    impl CommonSegment {
        pub fn text<T: Display>(content: T) -> Self {
            Self::Text(content.to_string())
        }

        pub fn image<T: Display>(url: T) -> Self {
            Self::Image(url.to_string())
        }

        pub fn img<T: Display>(url: T) -> Self {
            Self::image(url)
        }

        pub fn at<T: Display>(target: T) -> Self {
            Self::At(target.to_string())
        }

        #[must_use]
        pub const fn text_opt(&self) -> Option<&String> {
            match self {
                Self::Text(text) => Some(text),
                _ => None,
            }
        }
    }

    impl TryFrom<Segment> for CommonSegment {
        type Error = ValueError;

        fn try_from(value: Segment) -> Result<Self, Self::Error> {
            let Segment { ty, data } = value;
            match ty.as_str() {
                "text" => Ok(Self::Text(sithra_transport::from_value(data)?)),
                "image" => Ok(Self::Image(sithra_transport::from_value(data)?)),
                "at" => Ok(Self::At(sithra_transport::from_value(data)?)),
                _ => Ok(Self::Unknown(Segment { ty, data })),
            }
        }
    }

    impl From<CommonSegment> for Segment {
        fn from(value: CommonSegment) -> Self {
            match value {
                CommonSegment::Text(text) => Self::text(&text),
                CommonSegment::Image(image) => Self::image(&image),
                CommonSegment::At(target) => Self::at(&target),
                CommonSegment::Unknown(segment) => segment,
            }
        }
    }

    impl<'de> Deserialize<'de> for CommonSegment {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
            D::Error: de::Error,
        {
            let raw = Segment::deserialize(deserializer)?;
            raw.try_into().map_err(|_| D::Error::custom("Invalid segment"))
        }
    }

    impl Serialize for CommonSegment {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            let segment: Segment = self.clone().into();
            segment.serialize(serializer)
        }
    }
}

#[cfg(test)]
#[allow(unused)]
mod tests {
    use sithra_server::{
        extract::{
            context::{Clientful, Context as RawContext},
            payload::Payload,
            state::State,
        },
        handler::Handler,
        router,
        routing::router::Router,
        server::{Client, PostError},
    };
    use sithra_transport::channel::Channel;

    use super::Message;
    use crate::message::{ClientfulExt, ContextExt, SendMessage, common::CommonSegment};

    #[derive(Clone)]
    struct AppState {
        client: Client,
    }

    type Context<T> = RawContext<T, AppState>;

    impl Clientful for AppState {
        fn client(&self) -> &Client {
            &self.client
        }
    }

    async fn on_message(ctx: Context<Message>) -> Result<(), PostError> {
        let _msg: &Message = ctx.payload();
        ctx.reply(msg!(CommonSegment[
            text: &"Hello, world!",
            img: &"https://example.com/image.png"
        ]))
        .await?;
        Ok(())
    }

    async fn on_message2(channel: Channel, State(state): State<AppState>) -> Result<(), PostError> {
        state
            .send_message(
                channel,
                msg!(CommonSegment[
                    text: "Hello, world!",
                    img: "https://example.com/image.png"
                ]),
            )
            .await?;
        Ok(())
    }

    async fn on_message3(Payload(_msg): Payload<Message>) -> SendMessage {
        msg!(CommonSegment[
            text: "Hello, world!",
            img: "https://example.com/image.png"
        ])
        .into()
    }

    #[tokio::test]
    async fn _type() {
        let _router = router! { Router::new() =>
            Message[on_message, on_message2, on_message3]
        };
    }
}
