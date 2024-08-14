mod attrs;
mod yew_macro;

use proc_macro::TokenStream;
use quote::ToTokens;
use syn::parse_macro_input;

use crate::attrs::Attrs;

#[proc_macro_error::proc_macro_error]
#[proc_macro]
pub fn attrs(input: TokenStream) -> TokenStream {
    let root = parse_macro_input!(input as Attrs);
    TokenStream::from(root.into_token_stream())
}
