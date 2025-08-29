use std::sync::LazyLock;

use sithra_kit::{
    matchopt, plugin,
    server::extract::payload::Payload,
    types::{
        message::{Message, SendMessage, common::CommonSegment as H},
        msg,
    },
};

const WORDS: &str = include_str!("../static/words2.txt");
const FUNCTIONS: &str = include_str!("../static/functions.txt");
const STARTS: &str = include_str!("../static/starts.txt");
static WORDS_LEN: LazyLock<usize> = LazyLock::new(|| WORDS.lines().count());
static FUNCTIONS_LEN: LazyLock<usize> = LazyLock::new(|| FUNCTIONS.lines().count());
static STARTS_LEN: LazyLock<usize> = LazyLock::new(|| STARTS.lines().count());

#[tokio::main]
async fn main() {
    let (plugin, _) = plugin!();
    let plugin = plugin.map(|r| r.route_typed(Message::on(adachi)));
    log::info!("Dice plugin started");
    tokio::select! {
        _ = plugin.run().join_all() => {}
        _ = tokio::signal::ctrl_c() => {}
    }
}

async fn adachi(Payload(msg): Payload<Message<H>>) -> Option<SendMessage> {
    let number = matchopt!(msg.as_slice(), [H::Text(text)] => text.strip_prefix("adachi"))??;
    Some(msg!(rand_str(number.trim().parse().ok())))
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
