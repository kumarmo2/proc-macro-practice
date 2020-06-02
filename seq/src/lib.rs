extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::{Delimiter, Span, TokenStream as TokenStream2, TokenTree, Group, Literal};
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
        let content_token_stream;
        if lookahead.peek(token::Brace) {
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
    // TODO: refactor this into different method.
    let original: Vec<TokenTree> =  ds.content_token_stream.into_iter().collect();
    let start: u64 =  ds.lit_int_start.base10_parse().unwrap();
    let end: u64 = ds.lit_int_end.base10_parse().unwrap();
    println!("has repeated block: {:#?}", has_repeating_block(original.clone()));
    let has_repeating_block = has_repeating_block(original.clone());
    let mut result: Vec<TokenTree> = Vec::new();
    if has_repeating_block {
        let copied: Vec<TokenTree> = original.clone();
        let index = 0;
        let lit = LitInt::new(&format!("{}", index), Span::call_site());
        let tts: Vec<TokenTree> = replace_and_clone(&ds.counter_ident, &lit, copied, true);
        for token in tts {
            result.push(token);
        }
    } else {
        for index in start..end {
            let copied: Vec<TokenTree> = original.clone();
            let lit = LitInt::new(&format!("{}", index), Span::call_site());
            let tts: Vec<TokenTree> = replace_and_clone(&ds.counter_ident, &lit, copied, false);
            for token in tts {
                result.push(token);
            }
        }
    }
    let ts: TokenStream2 = TokenStream2::from_iter(result);

    println!("ts: {:#?}", ts);
    TokenStream::from(ts)
}


fn has_repeating_block(tree: Vec<TokenTree>) -> bool {
    let mut peekable_tree = tree.into_iter().peekable();
    while let Some(_) = peekable_tree.peek() {
        let token = peekable_tree.next().expect("=========first============");
        match token {
            TokenTree::Ident(_) | TokenTree::Literal(_) => {},
            TokenTree::Group(g) => {
                let stream: TokenStream2 = g.stream();
                let x: Vec<TokenTree> = stream.into_iter().collect();
                if let true = has_repeating_block(x) {
                    return true;
                }
                continue;
            },
            TokenTree::Punct(punct) => {
                if punct.as_char() != '#' {
                    continue;
                }
                if let None = peekable_tree.peek() {
                    panic!("expected token========= here");
                }
                let next_token = peekable_tree.next().expect("==============second=========");
                match next_token {
                    TokenTree::Group(g) => {
                        if g.delimiter() != Delimiter::Parenthesis {
                            let stream: TokenStream2 = g.stream();
                            let x: Vec<TokenTree> = stream.into_iter().collect();
                            if let true = has_repeating_block(x) {
                                return true;
                            }
                            continue;
                        } else {
                            if let None = peekable_tree.peek() {
                                panic!("=======expected token here");
                            } 
                            let next_next_token = peekable_tree.next().expect("================third");
                            // println!("next next token: {:#?}", next_next_token);
                            if let TokenTree::Punct(punct) = next_next_token {
                                if punct.as_char() != '*' {
                                    panic!("========== expected '*' token");
                                }
                                return true;

                            } else {
                                panic!("expect '*' token");
                            }
                        }
                    },
                    _ => {
                        continue;
                    }
                }
            }
        }
    }
    false
}

// TODO: Remove Vec clones if possible.
// Refactor.
// accept u64 instead of lit_int.
fn replace_and_clone(count_ident: &Ident, lit_int: &LitInt, tree: Vec<TokenTree>, has_repeating_block: bool) -> Vec<TokenTree>{
    let mut cloned: Vec<TokenTree> = Vec::new();
    let mut new_tree = tree.into_iter().peekable();
    while let Some(_) = new_tree.peek() {
        let token = new_tree.next().unwrap();
        match token {
            TokenTree::Ident(ident) => {
                if count_ident.to_string() == ident.to_string() {
                    let num = lit_int.base10_parse::<u64>().unwrap();
                    let lit = Literal::u64_unsuffixed(num);
                    cloned.push(TokenTree::Literal(lit));
                }else {
                    // Handling test-case 04.
                    if let Some(ref next_token) = new_tree.peek() {
                        if let TokenTree::Punct(p) = next_token {
                            if p.as_char() == '#' {
                                let _pound_token = new_tree.next().unwrap();
                                if let None = new_tree.peek() {
                                    panic!("expected a token here");
                                }
                                let c_tok = new_tree.next().unwrap();
                                if count_ident.to_string() ==  c_tok.to_string() {
                                    let num = lit_int.base10_parse::<u64>().unwrap();
                                    let lit = Literal::u64_unsuffixed(num);
                                    cloned.push(TokenTree::Ident(Ident::new(&format!("{}{}", ident.to_string(), lit.to_string()), Span::call_site())));
                                    continue;
                                }
                            }
                        }
                    }
                    cloned.push(TokenTree::Ident(ident));
                }
            },
            TokenTree::Punct(punct) => {
                if !has_repeating_block || punct.as_char() != '#'{
                    cloned.push(TokenTree::Punct(punct));
                    continue;
                }
                if let None = new_tree.peek() {
                    panic!("===========group token expected=======");
                }
                let next_token = new_tree.peek().expect("===========here========").clone();
                match next_token {
                    TokenTree::Group(group) => {
                        // new_tree.next().expect("should never happen");
                        if let Delimiter::Parenthesis = group.delimiter() {
                            new_tree.next().expect("should never happen"); // consuming the token.
                            // println!("===========found repeat group=======: {:#?}", group);
                            if let None = new_tree.peek() {
                                panic!("third expected '*' token");
                            }
                            let next_next_token = new_tree.next().expect("----sdfsdflsdfs-------");
                            match next_next_token {
                                TokenTree::Punct(punct) => {
                                    if punct.as_char() != '*' {
                                        panic!(" first expect '*' token here");
                                    }
                                    // println!("found *");
                                    // TODO: process the group.
                                    let stream: TokenStream2 = group.stream();
                                    let x: Vec<TokenTree> = stream.into_iter().collect();
                                    let new_tokens = replace_and_clone(count_ident, lit_int, x, has_repeating_block);
                                    cloned.extend(new_tokens.into_iter());
                                    // let ts = TokenStream2::from_iter(new);
                                    // let new_group = Group::new(group.delimiter(), ts);
                                    // cloned.push(TokenTree::Group(new_group));
                                }
                                _ => {
                                    // println!("====going to panic=======: next next token: {:#?}", next_next_token);
                                    panic!("second expect '*' token here");
                                }
                            }
                        } else {
                            cloned.push(TokenTree::Punct(punct));
                            continue;
                        }
                        // 1. if delimis are () and next next is punct(*)
                        // 2. if delimis are not () -> push # and the group as well.
                    },
                    _ => {
                        cloned.push(TokenTree::Punct(punct));
                        continue;
                    }
                }
            },
            TokenTree::Literal(lit) => {
                cloned.push(TokenTree::Literal(lit));
            },
            TokenTree::Group(group) => {
                let stream: TokenStream2 = group.stream();
                let x: Vec<TokenTree> = stream.into_iter().collect();
                let new = replace_and_clone(count_ident, lit_int, x, has_repeating_block);
                let ts = TokenStream2::from_iter(new);
                let new_group = Group::new(group.delimiter(), ts);
                cloned.push(TokenTree::Group(new_group));
            }
        }
    }
    cloned
}
