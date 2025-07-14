use sithra_kit::{
    server::extract::payload::Payload,
    types::{
        message::{Message, Segments, SendMessage, common::CommonSegment as H},
        smsg,
    },
};

pub async fn p1(Payload(msg): Payload<Message<H>>) -> Option<SendMessage> {
    let text = msg.content.first()?.text_opt()?;
    let text = text.strip_prefix("p1 ")?.to_owned();
    let Message { mut content, .. } = msg;
    {
        let first = content.first_mut()?;
        *first = H::text(&text);
    }
    let content: Segments<_> = content
        .into_iter()
        .map(|seg| {
            if let H::Text(v) = seg {
                let encrypted = v.trim().bytes().map(|b| b + 1);
                let encrypted = String::from_utf8(encrypted.collect());
                match encrypted {
                    Ok(encrypt) => H::text(encrypt),
                    Err(err) => H::text(err.to_string()),
                }
            } else {
                seg
            }
        })
        .collect();
    Some(smsg!(content))
}

pub async fn s1(Payload(msg): Payload<Message<H>>) -> Option<SendMessage> {
    let text = msg.content.first()?.text_opt()?;
    let text = text.strip_prefix("s1 ")?.to_owned();
    let Message { mut content, .. } = msg;
    {
        let first = content.first_mut()?;
        *first = H::text(&text);
    }
    let content: Segments<_> = content
        .into_iter()
        .map(|seg| {
            if let H::Text(v) = seg {
                let decrypted = v.trim().bytes().map(|b| b - 1);
                let decrypted = String::from_utf8(decrypted.collect());
                match decrypted {
                    Ok(decrypt) => H::text(decrypt),
                    Err(err) => H::text(err.to_string()),
                }
            } else {
                seg
            }
        })
        .collect();
    Some(smsg!(content))
}
