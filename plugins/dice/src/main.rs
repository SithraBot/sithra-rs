use std::{fmt::Display, str::FromStr};

use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, digit1},
    combinator::{eof, map_res, opt, value},
};
use serde::Deserialize;
use sithra_kit::{
    plugin,
    server::extract::{payload::Payload, state::State},
    types::{
        initialize::Initialize,
        message::{Message, SendMessage, common::CommonSegment as H},
        msg,
    },
};
use thiserror::Error;

#[derive(Deserialize, Clone, Default)]
struct Config {
    #[serde(rename = "expr-max-length", default)]
    expr_max_length: Option<usize>,
}

#[tokio::main]
async fn main() {
    let (plugin, Initialize { config, .. }) = plugin!(Option<Config>);
    let plugin =
        plugin.map(|r| r.route_typed(Message::on(dice)).with_state(config.unwrap_or_default()));
    log::info!("Dice plugin started");
    tokio::select! {
        _ = plugin.run().join_all() => {}
        _ = tokio::signal::ctrl_c() => {}
    }
}

async fn dice(
    Payload(msg): Payload<Message<H>>,
    State(Config { expr_max_length }): State<Config>,
) -> Option<SendMessage> {
    let dice = parse_dice(&msg.content)?;

    if let Err(err) = dice.verify() {
        return Some(msg!(f "{err}"));
    }

    log::debug!("Dice roll requested: {dice}");

    let Dice {
        face,
        times,
        select,
    } = dice;

    #[allow(clippy::cast_possible_truncation)]
    let mut results = Vec::with_capacity(times);
    for _ in 0..times {
        results.push(fastrand::u64(1..=face));
    }

    results.sort_unstable();

    let mut raw = if select.is_some() {
        let mut raw = results.iter().map(u64::to_string).collect::<Vec<String>>().join(", ");
        raw.push('\n');
        raw
    } else {
        String::new()
    };

    if let Some((select_high, select)) = select {
        if select_high {
            results.drain(..results.len() - select);
        } else {
            results.truncate(select);
        }
    }

    let mut expr = results.iter().map(u64::to_string).collect::<Vec<String>>().join(" + ");

    if expr_max_length.is_some_and(|eml| raw.len() > eml) {
        raw.clear();
    }
    if expr_max_length.is_some_and(|eml| expr.len() > eml) {
        expr.clear();
        expr.push_str("..");
    }

    let result = match results.as_slice() {
        [] => None,
        [first] => Some(format!("{first}")),
        _ => Some(format!("{raw}{expr} = {}", results.iter().sum::<u64>())),
    }?;

    Some(msg!(result))
}

struct Dice {
    face:   u64,
    times:  usize,
    /// (select high, select line)
    select: Option<(bool, usize)>,
}

impl Dice {
    const fn verify(&self) -> Result<(), DiceVerify> {
        if self.face == 0 {
            Err(DiceVerify::Face)
        } else if self.times == 0 {
            Err(DiceVerify::Times)
        } else if let Some((_, select)) = self.select {
            if select == 0 {
                Err(DiceVerify::Select)
            } else if select > self.times {
                Err(DiceVerify::SelectRange)
            } else {
                Ok(())
            }
        } else {
            Ok(())
        }
    }
}

impl Display for Dice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}d{}", self.face, self.times)?;
        if let Some((select_high, select)) = self.select {
            write!(f, "{}{}", if select_high { 'k' } else { 'q' }, select)?;
        }
        Ok(())
    }
}

#[derive(Debug, Error)]
enum DiceVerify {
    #[error("面数必须大于 0 喵")]
    Face,
    #[error("次数必须大于 0 喵")]
    Times,
    #[error("选择必须大于 0 喵")]
    Select,
    #[error("选择必须小于等于次数喵")]
    SelectRange,
}

fn parse_dice(msg: &[H]) -> Option<Dice> {
    let [H::Text(text), ..] = msg else {
        return None;
    };
    let ir: IResult<&str, _> = (
        map_res(digit1, u64::from_str),
        char('d'),
        alt((map_res(digit1, usize::from_str), value(1, tag("")))),
        opt((
            alt((value(true, char('k')), value(false, char('q')))),
            alt((map_res(digit1, usize::from_str), value(1, tag("")))),
        )),
        eof,
    )
        .parse(text);
    let (_, (face, _, times, select, _)) = ir.ok()?;
    Some(Dice {
        face,
        times,
        select,
    })
}
