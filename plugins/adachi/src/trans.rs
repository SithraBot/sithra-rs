use chrono::Utc;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use ulid::Ulid;

#[derive(Serialize)]
pub struct TransReq {
    q:         String,
    from:      String,
    to:        String,
    #[serde(rename = "appKey")]
    app_key:   String,
    salt:      String,
    sign:      String,
    #[serde(rename = "signType")]
    sign_type: String,
    curtime:   String,
    #[serde(rename = "vocabId")]
    vocab_id:  Option<String>,
}

impl TransReq {
    pub fn new(app_key: &str, app_secret: &str, q: &str, from: &str, to: &str) -> Self {
        let salt = Ulid::new().to_string();
        let curtime = Utc::now().timestamp();
        let q_char_len = q.chars().count();
        let input_for_sign = if q_char_len <= 20 {
            q.to_owned()
        } else {
            let first_10 = q.chars().take(10).collect::<String>();
            let last_10 =
                q.chars().rev().take(10).collect::<String>().chars().rev().collect::<String>();
            format!("{first_10}{q_char_len}{last_10}")
        };
        let payload = format!("{app_key}{input_for_sign}{salt}{curtime}{app_secret}");
        let mut hasher = Sha256::new();
        hasher.update(payload);
        let hash = hasher.finalize();
        let sign = format!("{hash:x}");

        Self {
            q: q.to_owned(),
            from: from.to_owned(),
            to: to.to_owned(),
            app_key: app_key.to_owned(),
            salt,
            sign,
            sign_type: String::from("v3"),
            curtime: curtime.to_string(),
            vocab_id: None,
        }
    }
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct TransRes {
    #[serde(rename = "errorCode")]
    pub error_code:  String,
    #[serde(default)]
    pub translation: Vec<String>,
    #[serde(default)]
    pub l:           String,
}

pub async fn post(req: TransReq) -> Result<TransRes, reqwest::Error> {
    let client = reqwest::Client::new();
    let response = client
        .post("https://openapi.youdao.com/api")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .form(&req)
        .send()
        .await?;
    response.json::<TransRes>().await
}

#[cfg(test)]
#[tokio::test]
async fn test() {
    let app_key = "";
    let app_secret = "";
    let q = "关于农业制度，关于附加文件…植物大战僵尸：辣椒的黄色王国";
    let from = "zh-CHS";
    let to = "en";

    let req = TransReq::new(app_key, app_secret, q, from, to);
    let result = post(req).await.unwrap();
    println!("{result:?}");
}
