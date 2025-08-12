use serde::{Deserialize, Serialize};
use sithra_kit::{
    transport::{
        channel::Channel,
        datapack::{DataPack, RequestDataPack},
    },
    types::{message::Message, smallvec::SmallVec},
};

use crate::{
    message::{OneBotSegment, internal::InternalOneBotSegment},
    util::de_str_from_num,
};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RawEvent {
    time:    u64,
    #[serde(deserialize_with = "de_str_from_num")]
    self_id: String,
    #[serde(flatten)]
    ty:      PostType,
}

impl RawEvent {
    #[must_use]
    pub fn channel(&self) -> Option<Channel> {
        match self.ty {
            PostType::Message(ref msg_event) => match msg_event.message_type {
                MessageEventKind::Group { ref group_id, .. } => Some(Channel::DirectFromGroup(
                    group_id.clone(),
                    msg_event.user_id.clone(),
                    msg_event.message_type.call_name(),
                )),
                MessageEventKind::Private { .. } => Some(Channel::Private(
                    msg_event.user_id.clone(),
                    msg_event.message_type.call_name(),
                )),
            },
            _ => None,
        }
        .map(|c| c.set_self_id(&self.self_id))
    }

    #[must_use]
    pub fn into_req(self, bot_id: &str) -> Option<DataPack> {
        let channel = self.channel();
        let Self {
            time: _,
            self_id: _,
            ty,
        } = self;
        match ty {
            PostType::Message(message_event) => {
                let message: Message = message_event.into();
                let req: RequestDataPack = RequestDataPack::default()
                    .path(Message::path())
                    .channel_opt(channel)
                    .bot_id(bot_id)
                    .payload(&message)
                    .ok()?;
                Some(req.into())
            }
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case", tag = "post_type")]
pub enum PostType {
    Message(MessageEvent),
    Notice,
    Request,
    MetaEvent,
    #[serde(other)]
    Unkonwn,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MessageEvent {
    #[serde(flatten)]
    pub message_type: MessageEventKind,
    #[serde(deserialize_with = "de_str_from_num")]
    pub message_id:   String,
    pub message:      SmallVec<[InternalOneBotSegment; 1]>,
    #[serde(deserialize_with = "de_str_from_num")]
    pub user_id:      String,
}

impl From<MessageEvent> for Message {
    fn from(value: MessageEvent) -> Self {
        Self {
            id:      value.message_id,
            content: value
                .message
                .into_iter()
                .filter_map(|segment| OneBotSegment(segment).try_into().ok())
                .collect(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case", tag = "message_type")]
pub enum MessageEventKind {
    Private {
        sender: PrivateSender,
    },
    Group {
        #[serde(deserialize_with = "de_str_from_num")]
        group_id: String,
        sender:   GroupSender,
    },
}

impl MessageEventKind {
    #[must_use]
    pub fn call_name(&self) -> String {
        match self {
            Self::Private { sender } => sender.nickname.clone(),
            Self::Group { sender, .. } => {
                if let Some(card) = &sender.card {
                    if card.is_empty() {
                        sender.nickname.clone()
                    } else {
                        card.clone()
                    }
                } else {
                    sender.nickname.clone()
                }
            }
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PrivateSender {
    nickname: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GroupSender {
    nickname: String,
    card:     Option<String>,
}
