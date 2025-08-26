#[macro_export]
/// A macro for concisely matching an expression against a pattern and returning
/// an `Option`.
///
/// This macro provides a convenient way to extract a value from an expression
/// if it matches a given pattern, optionally with a guard condition. If the
/// pattern matches (and the condition, if present, is true), it returns
/// `Some(value)`; otherwise, it returns `None`.
///
/// There are two forms of the macro:
///
/// 1. `matchopt!(expression, pattern [if condition] => value)`: This form
///    attempts to match `expression` against `pattern`. If successful, and if
///    an optional `if condition` is provided and evaluates to `true`, it
///    returns `Some(value)`. Otherwise, it returns `None`.
///
/// 2. `matchopt!(expression, pattern [if condition])`: This is a shorthand for
///    `matchopt!(expression, pattern [if condition] => ())`. It returns
///    `Some(())` if the pattern matches (and the condition, if present, is
///    true), and `None` otherwise. This is useful when you only care about
///    whether the match was successful, not about extracting a specific value.
///
/// # Examples
///
/// Basic usage with value extraction:
///
/// ```
/// use sithra_kit::matchopt;
/// 
/// let x = Some(5);
/// let y = matchopt!(x, Some(val) => val);
/// assert_eq!(y, Some(5));
///
/// let z = None::<i32>;
/// let w = matchopt!(z, Some(val) => val);
/// assert_eq!(w, None);
/// ```
///
/// Using a guard condition:
///
/// ```
/// use sithra_kit::matchopt;
/// 
/// let num = Some(10);
/// let even_num = matchopt!(num, Some(val) if val % 2 == 0 => val);
/// assert_eq!(even_num, Some(10));
///
/// let odd_num = Some(7);
/// let even_num = matchopt!(odd_num, Some(val) if val % 2 == 0 => val);
/// assert_eq!(even_num, None);
/// ```
///
/// Using the shorthand form (matching only):
///
/// ```
/// use sithra_kit::matchopt;
/// 
/// let result: Result<i32, &str> = Ok(42);
/// let is_ok = matchopt!(result, Ok(_));
/// assert_eq!(is_ok, Some(()));
///
/// let result: Result<i32, &str> = Err("error");
/// let is_ok = matchopt!(result, Ok(_));
/// assert_eq!(is_ok, None);
/// ```
///
/// Combining with guard condition in shorthand:
///
/// ```
/// use sithra_kit::matchopt;
/// 
/// let data = Some(vec![1, 2, 3]);
/// let has_elements = matchopt!(data, Some(v) if !v.is_empty());
/// assert_eq!(has_elements, Some(()));
///
/// let empty_data = Some(vec![]);
/// let has_elements = matchopt!(empty_data, Some(v) if !v.is_empty());
/// assert_eq!(has_elements, None);
/// ```
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
