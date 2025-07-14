use sithra_kit::types::message::{Segments, common::CommonSegment};

pub fn cmd(msg: &Segments<CommonSegment>) -> String {
    msg.iter().fold(String::new(), |f, s| {
        if let CommonSegment::Text(text) = s {
            f + text
        } else {
            f
        }
    })
}

pub fn truncate_tail(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        return s.to_owned();
    }

    if max_len < 2 {
        return s.chars().take(max_len).collect();
    }

    let take_chars = max_len - 2;
    let mut result = String::with_capacity(max_len);
    result.push_str(&s.chars().take(take_chars).collect::<String>());
    result.push_str("...");
    result
}
