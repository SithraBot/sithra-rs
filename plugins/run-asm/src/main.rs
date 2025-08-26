use std::pin::Pin;

use sithra_kit::{
    plugin,
    server::{
        extract::{botid::BotId, context::Clientful, payload::Payload, state::State},
        server::Client,
    },
    transport::channel::Channel,
    types::{
        message::{ClientfulExt, Message, SendMessage, common::CommonSegment as H},
        msg,
    },
};
use zkt_asm::machine::{InterruptHandler, Machine, MachineError, SharedMachine};

#[derive(Clone)]
struct AppState {
    machine: SharedMachine,
    client:  Client,
}
impl Clientful for AppState {
    fn client(&self) -> &Client {
        &self.client
    }
}

#[tokio::main]
async fn main() {
    let (plugin, _) = plugin!();
    let state = AppState {
        machine: Machine::shared(),
        client:  plugin.server.client(),
    };
    let plugin = plugin.map(|r| r.route_typed(Message::on(run)).with_state(state));
    log::info!("ZktASM plugin started");
    tokio::select! {
        _ = plugin.run().join_all() => {}
        _ = tokio::signal::ctrl_c() => {}
    }
}

async fn run(
    Payload(msg): Payload<Message<H>>,
    channel: Channel,
    State(AppState { machine, client }): State<AppState>,
    BotId(bot_id): BotId,
) -> Option<SendMessage> {
    let text = msg.content.first()?.text_opt()?;
    let text = text.strip_prefix("#!asm\n")?.to_owned();
    log::info!("recv: {text}");
    machine.register_interrupt_handler(
        1,
        SendMessageInterruptHandler::new(channel.clone(), client.clone(), bot_id.clone()),
    );
    machine.register_interrupt_handler(
        3,
        DebugRegMessageInterruptHandler::new(channel.clone(), client.clone(), bot_id.clone()),
    );
    machine.register_interrupt_handler(
        2,
        DebugMemMessageInterruptHandler::new(channel, client, bot_id),
    );
    let tokens = match zkt_asm::tokenizer::tokenize(&text) {
        Ok(tokens) => tokens,
        Err(err) => {
            log::error!("Failed to tokenize input: {err}");
            return Some(msg!(f "Failed to tokenize input: {err}"));
        }
    };
    if let Err(err) = machine.run(&tokens).await {
        log::error!("Failed to run: {err}");
        return Some(msg!(f "Failed to run: {err}"));
    }
    // Some(msg!(f "mem: {:?}", machine.read().unwrap().mem.as_slice()))
    None
}

macro_rules! interrupt {
    ($ident:ident => async |$m:ident, $ch:ident, $cl:ident, $b:ident| $block:block) => {
        struct $ident {
            channel: Channel,
            client:  Client,
            bot_id:  Option<String>,
        }

        impl $ident {
            const fn new(channel: Channel, client: Client, bot_id: Option<String>) -> Self {
                Self {
                    channel,
                    client,
                    bot_id,
                }
            }
        }

        impl InterruptHandler for $ident {
            type Future = Pin<Box<dyn Future<Output = Result<(), MachineError>> + Send + Sync>>;

            fn handle_interrupt(&self, $m: zkt_asm::machine::SharedMachineInner) -> Self::Future {
                let $ch = self.channel.clone();
                let $cl = self.client.clone();
                let $b = self.bot_id.clone();
                #[allow(clippy::cast_sign_loss)]
                #[allow(clippy::cast_possible_truncation)]
                Box::pin(async move $block)
            }
        }
    };
}

interrupt!(SendMessageInterruptHandler => async |m, channel, client, bot_id| {
    let addr = *m.read().unwrap().registers.get("r1").unwrap_or(&0) as usize;
    let mut bytes = Vec::new();
    let mut current_addr = addr;
    loop {
        let byte = m.read().unwrap().mem.get(current_addr)?;
        if byte == 0 {
            break;
        }
        bytes.push(byte);
        current_addr += 1;
    }
    let s = match String::from_utf8(bytes) {
        Ok(s) => s,
        Err(err) => return Err(MachineError::custom(err)),
    };
    log::info!("Sending message: {s}");
    if let Err(err) = client.send_message(channel, bot_id, &msg!(s)).await {
        log::error!("Failed to send message: {err}");
        return Err(MachineError::custom(err));
    }
    log::info!("Message sent successfully");
    Ok(())
});

interrupt!(DebugMemMessageInterruptHandler => async |m, channel, client, bot_id| {
    if let Err(err) = client
        .send_message(
            channel,
            bot_id,
            &msg!(f "{:?}", m.read().unwrap().mem.as_slice()),
        )
        .await
    {
        log::error!("Failed to send message: {err}");
        return Err(MachineError::custom(err));
    }
    Ok(())
});

interrupt!(DebugRegMessageInterruptHandler => async |m, channel, client, bot_id| {
    if let Err(err) = client
        .send_message(
            channel,
            bot_id,
            &msg!(f "{:?}", m.read().unwrap().registers),
        )
        .await
    {
        log::error!("Failed to send message: {err}");
        return Err(MachineError::custom(err));
    }
    Ok(())
});