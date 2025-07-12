use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::protocol::Message as WsMessage;

use crate::{api::response::ApiResponse, event::RawEvent};

pub mod api;
pub mod endpoint;
pub mod event;
pub mod message;
pub mod util;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum OneBotMessage {
    Event(RawEvent),
    Api(ApiResponse),
}

#[derive(Clone)]
pub struct AdapterState {
    pub ws_tx:               mpsc::UnboundedSender<WsMessage>,
    pub convert_file_base64: bool,
}

#[cfg(test)]
mod tests {
    use crate::{OneBotMessage, event::RawEvent};

    #[test]
    fn de() {
        let r = r#"{"message_type":"private","sub_type":"friend","message_id":62224923,"user_id":3605331714,"message":[{"type":"text","data":{"text":"aaaa"}}],"raw_message":"aaaa","font":0,"sender":{"user_id":3605331714,"nickname":"\u4FD7\u624B","sex":"unknown"},"target_id":1921576220,"message_style":{"bubble_id":0,"pendant_id":0,"font_id":0,"font_effect_id":0,"is_cs_font_effect_enabled":false,"bubble_diy_text_id":0},"time":1752007344,"self_id":1921576220,"post_type":"message"}"#;
        let _e: OneBotMessage = serde_json::from_str(r).unwrap();

        let r = r#"{"message_type":"group","sub_type":"normal","message_id":226242807,"group_id":754074796,"user_id":3065733051,"anonymous":null,"message":[{"type":"reply","data":{"id":"225925481"}},{"type":"at","data":{"qq":"2429618410","name":"@\u2716\uFE0F"}},{"type":"text","data":{"text":" *ore"}}],"raw_message":"[CQ:reply,id=225925481][CQ:at,qq=2429618410,name=@\u2716\uFE0F] *ore","font":0,"sender":{"user_id":3065733051,"nickname":"Liev_Amica","card":"\u8D85\u7EA7\u54C8\u6C14\u85AE\u4F7F\u7528\u9B54\u722A\u8C03\u6559gtnh\u0B67\u2362\u20DD\u0B68","sex":"unknown","age":0,"area":"","level":"95","role":"member","title":""},"message_style":{"bubble_id":2011915,"pendant_id":0,"font_id":0,"font_effect_id":0,"is_cs_font_effect_enabled":false,"bubble_diy_text_id":0},"time":1752034583,"self_id":1921576220,"post_type":"message"}"#;
        let _e: RawEvent = serde_json::from_str(r).unwrap();

        let r = r#"{"message_type":"group","sub_type":"normal","message_id":1684373655,"group_id":905311025,"user_id":191697786,"anonymous":null,"message":[{"type":"mface","data":{"url":"https://gxh.vip.qq.com/club/item/parcel/item/e9/e99c237b82636920cc5de7f29c08daca/raw300.gif","emoji_package_id":241144,"emoji_id":"e99c237b82636920cc5de7f29c08daca","key":"e779bd64f58c0c04","summary":"[\u75AF\u72C2\u6444\u5165]"}},{"type":"text","data":{"text":"[\u75AF\u72C2\u6444\u5165]"}}],"raw_message":"[CQ:mface]\u0026#91;\u75AF\u72C2\u6444\u5165\u0026#93;","font":0,"sender":{"user_id":191697786,"nickname":"Mo9uier","card":"","sex":"unknown","age":0,"area":"","level":"31","role":"member","title":""},"message_style":{"bubble_id":2086380,"pendant_id":0,"font_id":0,"font_effect_id":0,"is_cs_font_effect_enabled":false,"bubble_diy_text_id":0},"time":1752067348,"self_id":1921576220,"post_type":"message"}"#;

        let _e: RawEvent = serde_json::from_str(r).unwrap();
    }
}
