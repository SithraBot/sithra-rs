use sithra::{conf, loader};
use tokio::signal;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let config = conf::Config::load_config("./config.toml");
    let config = match config {
        Ok(config) => config,
        Err(err) => {
            log::error!("Failed to load config: {err}");
            return Err(err.into());
        }
    };
    let mut loader = loader::Loader::new(config);
    let errs = loader.load_all().await;
    for (name, err) in errs {
        log::error!("Failed to load plugin {name}: {err}");
    }

    signal::ctrl_c().await?;

    loader.abort_all();
    Ok(())
}
