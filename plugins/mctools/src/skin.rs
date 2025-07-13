use base64::{Engine, prelude::BASE64_STANDARD};
use sithra_kit::{
    server::extract::payload::Payload,
    types::{
        message::{Message, SendMessage, common::CommonSegment as H},
        msg,
    },
};

use crate::util::cmd;

async fn get_image(url: &str) -> Option<String> {
    match reqwest::get(url).await {
        Ok(response) => response.bytes().await.ok().map(|bytes| {
            let base64 = BASE64_STANDARD.encode(bytes);
            format!("base64://{base64}")
        }),
        Err(_) => None,
    }
}

async fn handle_mc_command(id: &str, endpoint: &str, error_message: &str) -> SendMessage {
    let id = id.trim();
    let url = format!("https://nmsr.nickac.dev/{endpoint}/{id}");
    let message = if let Some(image) = get_image(&url).await {
        msg!(H[img: &image])
    } else {
        msg!(H[text: &error_message])
    };
    message.into()
}

pub async fn mcbody(Payload(message): Payload<Message<H>>) -> Option<SendMessage> {
    Some(
        handle_mc_command(
            cmd(&message.content).strip_prefix("mcbody")?,
            "fullbody",
            "找不到你的皮肤喵。",
        )
        .await,
    )
}

pub async fn mchead(Payload(message): Payload<Message<H>>) -> Option<SendMessage> {
    Some(
        handle_mc_command(
            cmd(&message.content).strip_prefix("mchead")?,
            "head",
            "摸不着头脑喵。",
        )
        .await,
    )
}

pub async fn mcface(Payload(message): Payload<Message<H>>) -> Option<SendMessage> {
    Some(
        handle_mc_command(
            cmd(&message.content).strip_prefix("mcface")?,
            "face",
            "没脸喵，找不到喵。",
        )
        .await,
    )
}

pub async fn mcskin(Payload(message): Payload<Message<H>>) -> Option<SendMessage> {
    Some(
        handle_mc_command(
            cmd(&message.content).strip_prefix("mcskin")?,
            "skin",
            "找不到你的皮肤喵。",
        )
        .await,
    )
}
