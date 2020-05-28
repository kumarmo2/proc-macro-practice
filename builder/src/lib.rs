extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    // Prefix o_ means original
    // Prefix b_ means builder

    let ast: DeriveInput = parse_macro_input!(input as DeriveInput);
    let o_name = &ast.ident;

    let t = quote! {
        impl #o_name {
            pub fn builder() {}
        }
    };
    t.into()
}
