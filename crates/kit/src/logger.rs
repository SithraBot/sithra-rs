use log::Log;
use once_cell::sync::OnceCell;
use sithra_server::{server::ClientSink, transport::datapack::RequestDataPack};
use sithra_types::log::Log as LogRequest;

pub static LOGGER: OnceCell<ClientLogger> = OnceCell::new();

pub struct ClientLogger(ClientSink);

impl Log for ClientLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= log::Level::Info
    }

    fn log(&self, record: &log::Record) {
        let log_request = LogRequest::from(record);
        self.0.send(RequestDataPack::try_from(log_request)).ok();
    }

    fn flush(&self) {}
}

#[allow(clippy::missing_panics_doc)]
pub fn init_log(client_sink: ClientSink) {
    LOGGER.set(ClientLogger(client_sink)).ok();
    log::set_logger(LOGGER.get().expect("unreachable")).unwrap();
    log::set_max_level(log::LevelFilter::max());
}
