extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2, TokenTree};
use quote::{ToTokens, quote};
use std::result::Result;
use std::iter::FromIterator;
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
        // let token_tree;
        let content_token_stream;
        if lookahead.peek(token::Brace) {
            println!("inside if");
            let x: TokenTree = stream.parse()?;
            match x {
                TokenTree::Group(g) => {
                    content_token_stream = g.stream();
                },
                _ => {
                    panic!("Some error");
                }
            }
        } else {
            panic!("expected { token")
        }
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
    let tokens: Vec<TokenStream> = ds.content_token_stream.into_iter().enumerate().map(|(index, tok): (usize, TokenTree)| {
        println!("token: {:#?}", tok);
        let func_name_string = format!("my_func{}", index);
        let func_name_ident = Ident::new(&func_name_string, Span::call_site());
        let t = quote! {
            pub fn #func_name_ident(){}
        };
        t.into()
    }).collect();

    println!("tokens: {:#?}", tokens);
    TokenStream::from_iter(tokens)
}
