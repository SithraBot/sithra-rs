#[macro_export]
macro_rules! typed {
    ($route:expr => impl $typed:ty $([$($T:ident),*])?) => {
        // typed!(@private A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T,
        // U, V, W, X, Y, Z);
        #[allow(dead_code)]
        impl $(<$($T),*>)? $typed $(<$($T),*>)? {
            /// Create a new endpoint for the given route and handler.
            #[doc = concat!("Path: `", $route, "`\n\n")]
            /// Allowed payload:
            // $(#[doc = concat!(" - `", stringify!($T), "`\n")])*
            pub fn on<H, T, S>(
                handler: H,
            ) -> (
                &'static str,
                $crate::routing::endpoint::Endpoint<S, ::std::convert::Infallible>,
            )
            where
                H: $crate::handler::Handler<T, S>,
                T: 'static,
                S: ::std::clone::Clone + ::std::marker::Send + ::std::marker::Sync + 'static,
            {
                (
                    $route,
                    $crate::routing::endpoint::Endpoint::BoxedHandler(
                        $crate::boxed::BoxedIntoRoute::from_handler(handler),
                    ),
                )
            }

            #[doc(hidden)]
            pub fn __on<H, T, S>(
                handler: H,
            ) -> $crate::routing::endpoint::Endpoint<S, ::std::convert::Infallible>
            where
                H: $crate::handler::Handler<T, S>,
                T: 'static,
                S: ::std::clone::Clone + ::std::marker::Send + ::std::marker::Sync + 'static,
            {
                $crate::routing::endpoint::Endpoint::BoxedHandler(
                    $crate::boxed::BoxedIntoRoute::from_handler(handler),
                )
            }

            #[doc(hidden)]
            #[must_use]
            pub const fn path() -> &'static str {
                $route
            }

            #[doc(hidden)]
            pub const fn _check<H, T, S>(_handler: &H) -> &'static str
            where
                H: $crate::handler::Handler<T, S>,
                S: ::std::clone::Clone + ::std::marker::Send + ::std::marker::Sync + 'static,
            {
                $route
            }

            #[doc(hidden)]
            pub const fn __check<H, T, S>(handler: H) -> H
            where
                H: $crate::handler::Handler<T, S>,
                S: ::std::clone::Clone + ::std::marker::Send + ::std::marker::Sync + 'static,
            {
                handler
            }
        }
        // typed!(@default);
    }; /* (@private $first:ident $(, $rest:ident)*)=> {
        *     typed!(@inner $first $( ,$rest)*);
        *     typed!(@private $($rest),*);
        * };
        * (@private) => {
        *     impl AllowedPayload for () {}
        * };
        * (@inner $($T:ident),*)=> {
        *     impl<$($T,)*> AllowedPayload for ($($T),*,)
        *     where
        *         $($T: AllowedPayload  + 'static,)*
        *     {
        *     }
        * };
        * (@default) => {
        *     impl<Sta> AllowedPayload for $crate::extract::state::State<Sta> {}
        *     impl AllowedPayload for $crate::transport::channel::Channel {}
        *     impl AllowedPayload for $crate::extract::correlation::Correlation {}
        *     impl AllowedPayload for $crate::server::Client {}
        * }; */
}

#[macro_export]
macro_rules! on {
    ($handler:ident) => {
        ($crate::path_!($handler), $crate::on($handler))
    };
}

#[macro_export]
macro_rules! router {
    { $router:expr => $($typed:ty[$($handler:expr),*$(,)?]),*$(,)? } => {
        ($router)
        $(.route(
            <$typed>::path(),
            $crate::multi(
                [$(<$typed>::__on($handler),)*]
            )
        ))*
    }
}

#[cfg(test)]
fn _typed() {
    use crate::{
        extract::{payload::Payload, state::State},
        routing::{endpoint::Endpoint, router::Router},
    };

    mod message {
        pub struct Message;
        typed!("/message" => impl Message);
    }
    mod other {
        pub struct Other;
        typed!("/other" => impl Other);
    }

    async fn message_handler(Payload(_str): Payload<String>, State(()): State<()>) {}

    let _: (_, Endpoint<()>) = message::Message::on(async || {});
    let _: (_, Endpoint<()>) =
        message::Message::on(async |Payload(_str): Payload<String>, State(()): State<()>| {});
    let _: (_, Endpoint<()>) = message::Message::on(async |Payload(_str): Payload<String>| {});
    let _: (_, Endpoint<()>) = message::Message::on(async |State(_unit): State<()>| {});

    let router: Router = Router::new().route_typed(message::Message::on(async || {}));
    let _ = router! { router =>
        message::Message[message_handler, message_handler],
        other::Other[message_handler, async |State(_unit): State<()>| {}]
    };
}
