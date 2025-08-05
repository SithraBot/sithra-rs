use rune::{Diagnostics, Source, Sources};
use sithra_kit::{
    plugin,
    server::extract::{payload::Payload, state::State},
    types::{
        message::{Message, SendMessage, common::CommonSegment as H},
        smsg,
    },
};
use triomphe::Arc;

#[derive(Clone)]
pub struct AppState {
    context: Arc<rune::Context>,
    runtime: std::sync::Arc<rune::runtime::RuntimeContext>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let (plugin, _) = plugin!();
    let context = rune::Context::with_config(false)?;
    let runtime = context.runtime()?;
    let state = AppState {
        context: Arc::new(context),
        runtime: std::sync::Arc::new(runtime),
    };
    let plugin = plugin.map(move |r| r.route_typed(Message::on(run)).with_state(state));
    log::info!("Rune plugin started");
    tokio::select! {
        _ = plugin.run().join_all() => {}
        _ = tokio::signal::ctrl_c() => {}
    }
    Ok(())
}

async fn run(
    Payload(msg): Payload<Message<H>>,
    State(AppState { context, runtime }): State<AppState>,
) -> Option<SendMessage> {
    let text = msg.content.first()?.text_opt()?;
    let text = text.strip_prefix("run$\n")?.to_owned();
    let text = text.trim();
    if text.is_empty() {
        return Some(smsg!("我可执行不了空白代码喵"));
    }
    let source = match Source::memory(text) {
        Ok(source) => source,
        Err(err) => {
            log::warn!("Failed to create source: {err}");
            return Some(smsg!(format!("解析失败喵: \n{err}")));
        }
    };
    let mut sources = Sources::new();
    if let Err(err) = sources.insert(source) {
        log::warn!("Failed to insert source: {err}");
        return Some(smsg!(format!("插入源失败喵: \n{err}")));
    }
    let mut diagnostics = Diagnostics::new();
    let result = rune::prepare(&mut sources)
        .with_context(context.as_ref())
        .with_diagnostics(&mut diagnostics)
        .build();

    if !diagnostics.is_empty() {
        let mut writer = rune::termcolor::Ansi::new(Vec::new());
        if let Err(err) = diagnostics.emit(&mut writer, &sources) {
            log::error!("Failed to emit diagnostics: {err}");
            return None;
        }
        let output = String::from_utf8(writer.into_inner());
        match output {
            Ok(output) => {
                log::warn!("Diagnostics: {output}");
                // return Some(smsg!(format!("解析失败喵: \n{output}")));
            }
            Err(err) => {
                log::warn!("Failed to convert diagnostics output to string: {err}");
                return None;
            }
        }
    }

    let unit = match result {
        Ok(unit) => std::sync::Arc::new(unit),
        Err(err) => {
            log::error!("Failed to create source: {err}");
            return Some(smsg!(format!("解析失败喵: \n{err}")));
        }
    };

    let mut vm = rune::Vm::new(runtime, unit);

    let output = rune::alloc::limit::with(1024, || {
        let output = match vm.execute(["main"], ()) {
            Ok(mut output) => match output.complete() {
                rune::runtime::VmResult::Ok(output) => output,
                rune::runtime::VmResult::Err(err) => {
                    log::error!("Failed to complete output: {err}");
                    return format!("执行失败喵: \n{err}");
                }
            },
            Err(err) => {
                log::error!("Failed to execute program: {err}");
                return format!("执行失败喵: \n{err}");
            }
        };
        let raw = format!("{output:?}");
        if let Ok(str) = output.into_string() {
            str.to_string()
        } else {
            raw
        }
    })
    .call();

    Some(smsg!(output))
}
