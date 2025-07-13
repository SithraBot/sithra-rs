use resvg::usvg;
use sithra_kit::{plugin::Plugin, server::router, types::message::Message};
use triomphe::Arc;
mod server;
mod skin;
mod util;

const FONT: &[u8] = include_bytes!("../static/HarmonyOS_Sans_SC_Bold.ttf");

#[derive(Clone)]
pub struct AppState<'a> {
    pub svg_options: Arc<usvg::Options<'a>>,
    pub client:      reqwest::Client,
}

#[tokio::main]
async fn main() {
    let (plugin, _) = Plugin::new::<()>().await.unwrap();
    let mut svg_opt = usvg::Options::default();
    svg_opt.fontdb_mut().load_system_fonts();
    svg_opt.fontdb_mut().load_font_data(FONT.to_owned());

    let client = reqwest::Client::default();
    let state = AppState {
        svg_options: Arc::new(svg_opt),
        client,
    };
    let plugin = plugin.map(move |r| {
        router! { r =>
            Message [
                skin::mcbody,
                skin::mcface,
                skin::mchead,
                skin::mcskin,
                server::mcserver
            ]
        }
        .with_state(state)
    });
    log::info!("McTools plugin started");
    tokio::select! {
        _ = plugin.run().join_all() => {}
        _ = tokio::signal::ctrl_c() => {}
    }
}
