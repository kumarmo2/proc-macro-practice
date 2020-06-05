extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::Field;
use syn::Lit;
use syn::{parse_macro_input, Data, DeriveInput, Fields, Meta};

#[proc_macro_derive(CustomDebug, attributes(debug))]
pub fn derive(input: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(input as DeriveInput);
    // println!("input: {:#?}", derive_input);
    let name = derive_input.ident;
    let name_str = name.to_string();
    let fields;
    match derive_input.data {
        Data::Struct(ds) => match ds.fields {
            Fields::Named(named_fields) => {
                fields = named_fields;
            }
            _ => {
                panic!("Only named structs are allowed");
            }
        },
        _ => {
            panic!("currently it only supports struct");
        }
    }
    //TODO: can the call to clone() be cloned.
    let fields_format_vec: Vec<TokenStream2> = fields
        .named
        .clone()
        .into_iter()
        .map(|f| {
            let name_ident = f.ident.as_ref().expect("only named fields allowed.");
            let name_str = name_ident.to_string();
            let format_string = get_format_string_for_field(&f);
            println!("{:#?}", format_string);

            if let Some(fs) = format_string {
                println!("{}", fs);
                quote! {
                    .field(#name_str, &format!("{}", format_args!(#fs, &self.#name_ident)))
                }
            } else {
                quote! {
                    .field(#name_str, &self.#name_ident)
                }
            }
        })
        .collect();

    let tokens = quote! {
        use std::fmt::{Formatter, Error, Debug};

        impl Debug for #name {
            fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
                f.debug_struct(#name_str)
                    #(
                        #fields_format_vec
                    )*
                    .finish();
                Ok(())
            }
        }
    };
    tokens.into()
}

fn get_format_string_for_field(f: &Field) -> Option<String> {
    // println!("=====field:{:#?}", f);
    for attr in &f.attrs {
        match attr.parse_meta() {
            Ok(meta) => {
                // println!("meta: {:#?}", meta);
                let meta_nv;
                match meta {
                    Meta::NameValue(nv) => {
                        meta_nv = nv;
                    }
                    _ => {
                        return None;
                    }
                }
                let debug_attr = meta_nv
                    .path
                    .segments
                    .iter()
                    .find(|seg| seg.ident.to_string() == "debug");
                let literal_val;
                if let Some(_) = debug_attr {
                    literal_val = meta_nv.lit;
                } else {
                    return None;
                }
                if let Lit::Str(lit_str) = literal_val {
                    return Some(lit_str.value());
                } else {
                    return None;
                }
            }
            Err(_) => {
                // println!("could not be prsed");
            }
        }
    }
    None
}
