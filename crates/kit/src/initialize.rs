#[doc(hidden)]
pub mod __private {
    pub use futures_util::StreamExt;
}

#[deprecated]
#[macro_export]
macro_rules! init {
    ($peer:expr, $config:ty) => {{
        let mut framed = $crate::transport::util::framed($peer);

        let config = loop {
            let Some(msg) = <$crate::transport::util::FramedPeer as $crate::initialize::__private::StreamExt>::next(&mut framed).await else {
                break Err("Connection closed".to_owned());
            };
            if let Ok(msg) = msg {
                let is_init = msg.path.as_ref().is_some_and(|p| p == $crate::types::initialize::Initialize::<$config>::path());
                if is_init {
                    let config = msg.payload::<$config>();
                    break config;
                }
            }
        };

        let peer = framed.into_inner();
        (peer, config)
    }};
}
