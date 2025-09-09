use std::sync::LazyLock;

use serde::Deserialize;
use simple_ai::OnceChat;
use sithra_kit::{
    matchopt, plugin,
    server::{
        extract::context::{Clientful, Context},
        server::Client,
    },
    types::{
        initialize::Initialize,
        message::{Message, SendMessage, common::CommonSegment as H},
        msg,
    },
};

use crate::trans::{TransReq, post};

mod trans;

const WORDS: &str = include_str!("../static/words.txt");
const FUNCTIONS: &str = include_str!("../static/functions.txt");
const STARTS: &str = include_str!("../static/starts.txt");
static WORDS_LEN: LazyLock<usize> = LazyLock::new(|| WORDS.lines().count());
static FUNCTIONS_LEN: LazyLock<usize> = LazyLock::new(|| FUNCTIONS.lines().count());
static STARTS_LEN: LazyLock<usize> = LazyLock::new(|| STARTS.lines().count());

#[derive(Deserialize, Clone, Default)]
#[serde(default)]
struct Config {
    youdao: Option<YoudaoAPI>,
    #[serde(default)]
    use_ai: bool,
}

#[derive(Deserialize, Clone)]
struct YoudaoAPI {
    app_key:    String,
    app_secret: String,
}

#[derive(Clone)]
struct AppState {
    config: Config,
    client: Client,
}
impl Clientful for AppState {
    fn client(&self) -> &Client {
        &self.client
    }
}

#[tokio::main]
async fn main() {
    let (plugin, Initialize { config, .. }) = plugin!(Config);
    let client = plugin.server.client();
    let plugin =
        plugin.map(|r| r.route_typed(Message::on(adachi)).with_state(AppState { config, client }));
    log::info!("Adachi plugin started");
    tokio::select! {
        _ = plugin.run().join_all() => {}
        _ = tokio::signal::ctrl_c() => {}
    }
}

async fn adachi(ctx: Context<Message<H>, AppState>) -> Option<SendMessage> {
    let number = matchopt!(ctx.as_slice(), [H::Text(text)] => text.strip_prefix("adachi"))??;
    let randstr = rand_str(number.trim().parse().ok());
    if ctx.state.config.use_ai {
        let data = OnceChat(format!(
            "Please correct the grammar of the following sentence and output only the corrected \
             sentence, without any additional explanations: {randstr}"
        ));
        let response = ctx.post(data).await.ok()?;
        return Some(msg!(response));
    }
    let randstr = if let Some(youdao) = ctx.state.config.youdao {
        let req = TransReq::new(&youdao.app_key, &youdao.app_secret, &randstr, "auto", "en");
        let result_en = post(req).await.ok()?;
        let req = TransReq::new(
            &youdao.app_key,
            &youdao.app_secret,
            &result_en.translation.join("\n"),
            "en",
            "zh-CHS",
        );
        let result_zh = post(req).await.ok()?;
        let result = result_zh.translation.join("\n").trim().to_owned();
        if result.is_empty() { randstr } else { result }
    } else {
        randstr
    };
    Some(msg!(randstr))
}

fn rand_str(len: Option<usize>) -> String {
    let mut rng = fastrand::Rng::new();
    let main_len = if let Some(len) = len {
        len
    } else {
        rng.usize(2..=4)
    };
    let function_len = main_len - 1;
    let mut s = String::with_capacity(main_len * 3 + function_len * 3);
    let mut main_nths = Vec::with_capacity(main_len);
    let mut function_nths = Vec::with_capacity(function_len);
    for _ in 0..main_len {
        main_nths.push(rng.usize(0..*WORDS_LEN));
    }
    for _ in 0..function_len {
        function_nths.push(rng.usize(0..*FUNCTIONS_LEN));
    }
    let mut main_nths = main_nths.into_iter();
    let mut function_nths = function_nths.into_iter();
    if let Some(start) = STARTS.lines().nth(rng.usize(0..*STARTS_LEN)) {
        s.push_str(start);
    }
    while let (Some(main_nth), function_nth) = (main_nths.next(), function_nths.next()) {
        let line = WORDS.lines().nth(main_nth).unwrap();
        s.push_str(line);
        if let Some(function_nth) = function_nth {
            let function_line = FUNCTIONS.lines().nth(function_nth).unwrap();
            s.push_str(function_line);
        }
    }
    s
}

#[cfg(test)]
#[test]
fn rand_str_test() {
    use std::io::Write;
    let file = std::fs::File::create("test.txt").unwrap();
    let mut writer = std::io::BufWriter::new(file);
    for _ in 0..100 {
        let result = rand_str(None);
        writeln!(writer, "{result}").unwrap();
    }
}
