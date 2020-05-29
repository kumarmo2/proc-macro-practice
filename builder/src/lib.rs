extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Type};

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    // Prefix o_ means original
    // Prefix b_ means builder

    // TODO: Currently it will work for only Structs with named fields.

    use syn::{Data, DataStruct, Fields, FieldsNamed, Ident};

    let ast: DeriveInput = parse_macro_input!(input as DeriveInput);
    let o_name = &ast.ident;
    let b_name = format!("{}Builder", o_name);
    let b_ident = Ident::new(&b_name, o_name.span());

    // let x = Some(32);

    let fields = if let Data::Struct(DataStruct {
        fields: Fields::Named(FieldsNamed { named, .. }),
        ..
    }) = ast.data
    {
        named
    } else {
        unimplemented!();
    };

    let optionized_fields = fields.iter().map(|f| {
        let name = &f.ident.as_ref().unwrap();
        let ty = &f.ty;
        match is_field_option_type(f) {
            Some(_) => {
                quote! {
                    #name: #ty,
                }
            }
            None => {
                quote! {
                    #name: Option<#ty>,
                }
            }
        }
    });

    let setters = fields.iter().map(|f| {
        let name = &f.ident;
        let mut ty = &f.ty;
        match is_field_option_type(f) {
            Some(t) => ty = t,
            None => {}
        }

        quote! {
            fn #name(&mut self, #name: #ty) -> &mut #b_ident {
                self.#name = Some(#name);
                self
            }
        }
    });

    let builder_default_fields = fields.iter().map(|f| {
        let name = &f.ident;
        quote! {
            #name: None,
        }
    });

    let build_fields = fields.iter().map(|f| {
        let name = &f.ident;
        match is_field_option_type(f) {
            Some(_) => {
                quote! {
                    #name: self.#name.clone(),
                }
            }
            None => quote! {
                #name: self.#name.clone().ok_or("sdfsdf")?,
            },
        }
    });

    let t = quote! {
        // use std::opt::Option;
        impl #o_name {
            pub fn builder() -> #b_ident{
                #b_ident{
                    #(
                        #builder_default_fields
                    )*
                }
            }
        }

        impl #b_ident {
            #(
                #setters
            )*
        }

        impl #b_ident {
        pub fn build(&mut self) -> Result<#o_name, Box<dyn std::error::Error>> {
            Ok(#o_name {
                #(
                    #build_fields
                )*
            })
        }

        }

        pub struct #b_ident {
            #(#optionized_fields)*
        }
    };
    t.into()
}

// TODO: Refactor this method. Damn its soo unreadable! -_-
fn is_field_option_type<'a>(f: &'a syn::Field) -> Option<&'a Type> {
    use syn::{AngleBracketedGenericArguments, GenericArgument, Path, PathArguments, TypePath};
    let ty = &f.ty;
    match ty {
        Type::Path(type_path) => {
            if let TypePath {
                path: Path { segments, .. },
                ..
            } = type_path
            {
                for segment in segments {
                    let ident = &segment.ident;
                    if ident.to_string() == "Option".to_string() {
                        match &segment.arguments {
                            PathArguments::AngleBracketed(AngleBracketedGenericArguments {
                                args,
                                ..
                            }) => match args.first().unwrap() {
                                GenericArgument::Type(t) => {
                                    return Some(t);
                                }
                                _ => {
                                    return None;
                                }
                            },
                            _ => return None,
                        }
                    }
                    return None;
                }
                return None;
            }
            return None;
        }
        _ => None,
    }
}
