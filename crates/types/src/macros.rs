#[macro_export]
#[doc(hidden)]
macro_rules! into_response {
    ($path:expr, $type:ty) => {
        impl $crate::__private::sithra_server::response::IntoResponse for $type {
            fn into_response(self) -> $crate::__private::sithra_server::response::Response {
                $crate::__private::sithra_transport::datapack::RequestDataPack::default()
                    .path($path)
                    .payload(self)
                    .into_response()
            }
        }
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! into_request {
    ($path:expr, $type:ty) => {
        impl ::std::convert::From<$type>
            for $crate::__private::sithra_transport::datapack::RequestDataPack
        {
            fn from(value: $type) -> Self {
                Self::default().path($path).payload(value)
            }
        }
    };
}

#[macro_export]
macro_rules! map {
    {$($key:tt: $value:expr),*} => {
        $crate::__private::rmpv::Value::Map(::std::vec![
            $((
                ::core::convert::Into::<$crate::__private::rmpv::Value>::into($key),
                ::core::convert::Into::<$crate::__private::rmpv::Value>::into($value)
            )),*
        ])
    };
}
