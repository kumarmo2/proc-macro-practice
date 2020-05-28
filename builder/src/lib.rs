extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

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
    println!("builder name: {}", b_name);
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

        quote! {
            #name: Option<#ty>,
        }
    });

    let setters = fields.iter().map(|f| {
        let name = &f.ident;
        let ty = &f.ty;

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

    let t = quote! {
        impl #o_name {
            pub fn builder() -> #b_ident{
            // pub fn builder() -> Self{
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

        pub struct #b_ident {
            #(#optionized_fields)*
        }
    };
    t.into()
}
