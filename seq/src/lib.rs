extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens, quote};
use std::result::Result;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, token, Error, Ident, LitInt, Token,
};

#[derive(Debug)]
struct Ds {
    counter_ident: Ident,
    lit_int_start: LitInt,
    lit_int_end: LitInt,
    content_token_stream: TokenStream2,
}

impl Parse for Ds {
    fn parse(stream: ParseStream) -> Result<Self, Error> {
        let counter_ident = stream.parse::<Ident>()?;
        stream.parse::<Token![in]>()?;
        let lit_int_start = stream.parse::<LitInt>()?;
        stream.parse::<Token![..]>()?;
        let lit_int_end = stream.parse::<LitInt>()?;
        let lookahead = stream.lookahead1();
        let content_token_stream;
        if lookahead.peek(token::Brace) {
            println!("inside if");
            let x: syn::Expr = stream.parse()?;
            content_token_stream = x.to_token_stream();
        } else {
            panic!("expected { token")
        }
        // let content_token_stream = content.to_token_stream();
        Ok(Ds {
            counter_ident,
            lit_int_start,
            lit_int_end,
            content_token_stream,
        })
    }
}

#[proc_macro]
pub fn seq(input: TokenStream) -> TokenStream {
    let ds = parse_macro_input!(input as Ds);
    let content = &ds.content_token_stream;
    // parse_macro_input!(input as Ds);
    // println!("ds: {:#?}", ds);
    // TokenStream::new()
    let t = quote! {
        #content
        // pub fn my_fun(){}
    };
    t.into()
}
