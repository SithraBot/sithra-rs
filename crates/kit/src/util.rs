#[macro_export]
macro_rules! matchopt {
    ($expr:expr, $pat:pat $(if $cond:expr)? => $var:expr) => {
        match $expr {
            $pat $(if $cond)* => Some($var),
            _ => None,
        }
    };
    ($expr:expr, $pat:pat $(if $cond:expr)?) => {
        $crate::matchopt!($expr, $pat $(if $cond)* => ())
    };
}
