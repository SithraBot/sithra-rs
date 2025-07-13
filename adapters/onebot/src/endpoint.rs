#![allow(clippy::unused_async)]
use serde_json::json;
use sithra_kit::{
    server::{
        extract::{correlation::Correlation, payload::Payload, state::State},
        response::Response,
    },
    transport::channel::Channel,
    types::{
        channel::SetMute,
        message::{Segments, SendMessage},
    },
};

use crate::{AdapterState, api::request::ApiCall, message::OneBotSegment, util::send_req};

pub async fn send_message(
    Payload(payload): Payload<SendMessage>,
    State(state): State<AdapterState>,
    Correlation(id): Correlation,
    channel: Channel,
) -> Option<Response> {
    let segments = payload.content.into_iter().filter_map(|s| match OneBotSegment::try_from(s) {
        Ok(segment) => match segment {
            OneBotSegment(segment) => Some(segment),
        },
        Err(_err) => None,
    });
    let segments: Segments<_> = if state.convert_file_base64 {
        let mut result = Segments::new();
        for seg in segments {
            result.push(seg.or_in_base64().await);
        }
        result
    } else {
        segments.collect()
    };
    let req = if let Some(group_id) = channel.parent_id {
        ApiCall::new(
            "send_msg",
            json!({
                "message_type": "group",
                "group_id": group_id,
                "message": segments
            }),
            id,
        )
    } else {
        ApiCall::new(
            "send_msg",
            json!({
                "message_type": "private",
                "user_id": channel.id,
                "message": segments
            }),
            id,
        )
    };
    send_req(&state, id, &req, "send_msg")
}

pub async fn set_mute(
    Payload(payload): Payload<SetMute>,
    State(state): State<AdapterState>,
    Correlation(id): Correlation,
) -> Option<Response> {
    let SetMute { channel, duration } = payload;
    let Channel {
        id: user_id,
        ty: _,
        name: _,
        parent_id,
        self_id: _,
    } = channel;
    let Some(parent_id) = parent_id else {
        log::error!("Set Mute Failed to get parent_id");
        let mut response = Response::error("Failed to get parent_id");
        response.correlate(id);
        return Some(response);
    };
    let duration = duration.as_secs();
    let req = ApiCall::new(
        "set_group_ban",
        json!({
            "group_id": parent_id,
            "user_id": user_id,
            "duration": duration
        }),
        id,
    );
    send_req(&state, id, &req, "set_mute")
}
