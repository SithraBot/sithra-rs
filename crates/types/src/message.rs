use serde::{Deserialize, Serialize};
use sithra_server::{
    extract::context::{Clientful, Context},
    server::PostError,
};
use sithra_transport::{channel::Channel, datapack::RequestDataPack};
use smallvec::SmallVec;
use typeshare::typeshare;

pub const NIL: rmpv::Value = rmpv::Value::Nil;

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
    pub data: rmpv::Value,
}

impl Segment {
    pub fn text<T: ToString>(content: &T) -> Self {
        Self {
            ty:   "text".to_owned(),
            data: content.to_string().into(),
        }
    }

    pub fn image<T: ToString>(url: &T) -> Self {
        Self {
            ty:   "image".to_owned(),
            data: url.to_string().into(),
        }
    }

    pub fn img<T: ToString>(url: &T) -> Self {
        Self::image(url)
    }

    pub fn at<T: ToString>(target: &T) -> Self {
        Self {
            ty:   "at".to_owned(),
            data: target.to_string().into(),
        }
    }

    /// # Errors
    pub fn custom<T: ToString, V: Serialize>(ty: &T, data: V) -> Result<Self, rmpv::ext::Error> {
        Ok(Self {
            ty:   ty.to_string(),
            data: rmpv::ext::to_value(data)?,
        })
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

impl<Seg: Into<Segment>> From<SmallVec<[Seg; 1]>> for SendMessage {
    fn from(content: SmallVec<[Seg; 1]>) -> Self {
        Self {
            content: content.into_iter().map(Into::into).collect(),
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
    pub const PATH: &str = "/event/message.created";

    use super::Message;
    typed!("/event/message.created" => impl Message);
}

pub mod command {
    use sithra_server::typed;

    use super::SendMessage;
    use crate::into_response;
    typed!("/command/message.create" => impl SendMessage);

    into_response!("/command/message.create", SendMessage);
}

pub mod common {
    use de::Error as _;
    use serde::{Deserialize, Serialize, de};

    use crate::message::Segment;

    #[derive(Debug, Clone)]
    pub enum CommonSegment {
        Text(String),
        Image(String),
        At(String),
        Unknown(Segment),
    }

    impl CommonSegment {
        pub fn text<T: ToString>(content: &T) -> Self {
            Self::Text(content.to_string())
        }

        pub fn image<T: ToString>(url: &T) -> Self {
            Self::Image(url.to_string())
        }

        pub fn img<T: ToString>(url: &T) -> Self {
            Self::image(url)
        }

        pub fn at<T: ToString>(target: &T) -> Self {
            Self::At(target.to_string())
        }
    }

    impl TryFrom<Segment> for CommonSegment {
        type Error = rmpv::ext::Error;

        fn try_from(value: Segment) -> Result<Self, Self::Error> {
            let Segment { ty, data } = value;
            match ty.as_str() {
                "text" => Ok(Self::Text(rmpv::ext::from_value(data)?)),
                "image" => Ok(Self::Image(rmpv::ext::from_value(data)?)),
                "at" => Ok(Self::At(rmpv::ext::from_value(data)?)),
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
                    text: &"Hello, world!",
                    img: &"https://example.com/image.png"
                ]),
            )
            .await?;
        Ok(())
    }

    async fn on_message3(Payload(_msg): Payload<Message>) -> SendMessage {
        msg!(CommonSegment[
            text: &"Hello, world!",
            img: &"https://example.com/image.png"
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
