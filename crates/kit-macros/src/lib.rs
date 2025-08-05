// use proc_macro::TokenStream;
// use quote::{format_ident, quote};
// use syn::{
//     ItemFn, Result, Type,
//     parse::{Parse, ParseStream},
//     parse_macro_input,
// };

// struct AttributeArgs {
//     ty: Type,
// }

// impl Parse for AttributeArgs {
//     fn parse(input: ParseStream) -> Result<Self> {
//         let ty = input.parse::<Type>()?;
//         Ok(AttributeArgs { ty })
//     }
// }

// /// Attribute macro that registers a handler function with a typed route.
// ///
// /// # Example
// /// ```
// /// #[on(Message)]
// /// async fn message_handler(
// ///     Payload(str): Payload<String>,
// ///     State(()): State<()>,
// /// ) {
// ///     // handler implementation
// /// }
// /// ```
// ///
// /// This macro generates a constant that contains the route path for the
// /// handler.
// #[proc_macro_attribute]
// pub fn on(args: TokenStream, input: TokenStream) -> TokenStream {
//     let args = parse_macro_input!(args as AttributeArgs);
//     let input = parse_macro_input!(input as ItemFn);

//     let ty = &args.ty;

//     let vis = input.vis.clone();
//     let fn_name = &input.sig.ident;
//     let const_name = format_ident!("{fn_name}__path__");

//     let output = quote! {
//         #input

//         #[doc(hidden)]
//         #vis const #const_name: &'static str = #ty::_check(&#fn_name);
//     };

//     output.into()
// }

// #[proc_macro]
// pub fn path_(input: TokenStream) -> TokenStream {
//     let input_str = input.to_string();
//     let output_str = format!("{input_str}__path__");

//     match output_str.parse() {
//         Ok(tokens) => tokens,
//         Err(e) => {
//             let error = format!("Failed to parse generated string
// `{output_str}`: {e}");             TokenStream::from(
//                 syn::Error::new(proc_macro2::Span::call_site(),
// error).to_compile_error(),             )
//         }
//     }
// }
