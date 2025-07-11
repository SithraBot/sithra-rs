use sithra_kit::{plugin::Plugin, server::router, types::message::Message};
mod skin;

#[tokio::main]
async fn main() {
    let (plugin, _) = Plugin::new::<()>().await.unwrap();
    let plugin = plugin.map(|r| {
        router! { r =>
            Message [
                skin::mcbody,
                skin::mcface,
                skin::mchead,
                skin::mcskin
            ]
        }
    });
    log::info!("McTools plugin started");
    tokio::select! {
        _ = plugin.run().join_all() => {}
        _ = tokio::signal::ctrl_c() => {}
    }
}
