use std::fmt::Display;

use serde::{Deserialize, Serialize};
use sithra_transport::{Value, ValueError};

#[derive(Deserialize, Serialize)]
pub struct Initialize<C> {
    pub config:    C,
    pub id:        String,
    pub data_path: String,
}

impl<C> Initialize<C> {
    pub fn new<D1: Display, D2: Display>(config: C, name: D1, data_path: D2) -> Self {
        Self {
            config,
            id: name.to_string(),
            data_path: data_path.to_string(),
        }
    }
}

impl<C> Initialize<C>
where
    C: for<'de> Deserialize<'de>,
{
    /// # Errors
    /// Returns an error if the provided value cannot be deserialized into the
    /// config type.
    pub fn from_value(value: Value) -> Result<Self, ValueError> {
        let this = sithra_transport::from_value(value)?;
        Ok(this)
    }
}

pub mod command {
    use super::Initialize;

    #[allow(dead_code)]
    impl<C> Initialize<C> {
        /// Create a new endpoint for the given route and handler.
        #[doc = concat!("Path: `","/initialize","`\n\n")]
        /// Allowed payload:
        pub fn on<H, T, S>(
            handler: H,
        ) -> (
            &'static str,
            sithra_server::routing::endpoint::Endpoint<S, ::std::convert::Infallible>,
        )
        where
            H: sithra_server::handler::Handler<T, S>,
            T: 'static,
            S: ::std::clone::Clone + ::std::marker::Send + ::std::marker::Sync + 'static,
        {
            (
                "/initialize",
                sithra_server::routing::endpoint::Endpoint::BoxedHandler(
                    sithra_server::boxed::BoxedIntoRoute::from_handler(handler),
                ),
            )
        }

        #[doc(hidden)]
        pub fn __on<H, T, S>(
            handler: H,
        ) -> sithra_server::routing::endpoint::Endpoint<S, ::std::convert::Infallible>
        where
            H: sithra_server::handler::Handler<T, S>,
            T: 'static,
            S: ::std::clone::Clone + ::std::marker::Send + ::std::marker::Sync + 'static,
        {
            sithra_server::routing::endpoint::Endpoint::BoxedHandler(
                sithra_server::boxed::BoxedIntoRoute::from_handler(handler),
            )
        }

        #[doc(hidden)]
        #[must_use]
        pub const fn path() -> &'static str {
            "/initialize"
        }

        #[doc(hidden)]
        pub const fn _check<H, T, S>(_handler: &H) -> &'static str
        where
            H: sithra_server::handler::Handler<T, S>,
            S: ::std::clone::Clone + ::std::marker::Send + ::std::marker::Sync + 'static,
        {
            "/initialize"
        }

        #[doc(hidden)]
        pub const fn __check<H, T, S>(handler: H) -> H
        where
            H: sithra_server::handler::Handler<T, S>,
            S: ::std::clone::Clone + ::std::marker::Send + ::std::marker::Sync + 'static,
        {
            handler
        }
    }
}
