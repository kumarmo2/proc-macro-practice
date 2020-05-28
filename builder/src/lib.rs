extern crate proc_macro;
use proc_macro::TokenStream;
#[proc_macro_derive(Builder)]
pub fn derive(_: TokenStream) -> TokenStream {
    // Prefix o_ means original
    // Prefix b_ means builder
    TokenStream::new()
}
