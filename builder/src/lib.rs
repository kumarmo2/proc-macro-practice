extern crate proc_macro;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::Ident;
use syn::{
    parse_macro_input, DeriveInput, Field, GenericArgument, Lit, Meta, NestedMeta, PathArguments,
    Type,
};

#[proc_macro_derive(Builder, attributes(builder))]
pub fn derive(input: TokenStream) -> TokenStream {
    // Prefix o_ means original
    // Prefix b_ means builder

    // TODO: Currently it will work for only Structs with named fields.

    use syn::{Data, DataStruct, Fields, FieldsNamed};

    let ast: DeriveInput = parse_macro_input!(input as DeriveInput);
    let o_name = &ast.ident;
    let b_name = format!("{}Builder", o_name);
    let b_ident = Ident::new(&b_name, o_name.span());

    let fields = if let Data::Struct(DataStruct {
        fields: Fields::Named(FieldsNamed { named, .. }),
        ..
    }) = ast.data
    {
        named
    } else {
        // Only supports structs.
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
            None => match is_builder_of(f) {
                None => {
                    quote! {
                        #name: std::option::Option<#ty>,
                    }
                }
                Some(_) => {
                    quote! {
                        #name: #ty,
                    }
                }
            },
        }
    });

    let setters = fields.iter().map(|f| {
        let name = &f.ident;
        let mut ty = &f.ty;
        match is_field_option_type(f) {
            Some(t) => ty = t,
            None => {}
        }

        let builder_method_name = is_builder_of(f);

        match builder_method_name {
            None => {
                quote! {
                    fn #name(&mut self, #name: #ty) -> &mut #b_ident {
                        self.#name = Some(#name);
                        self
                    }
                }
            }
            Some(method_name) => {
                if name.as_ref().unwrap().to_string() == method_name {
                    // println!("builder method with same name");
                    return quote! {
                        fn #name(&mut self, #name: #ty) -> &mut #b_ident {
                            // self.#name = Some(#name);
                            self.#name = #name;
                            self
                        }
                    };
                } else {
                    // Assuming the field is Vec<T>
                    match get_generic_type_of_vec(f) {
                        Some(gen_ident) => {
                            let new_method_ident = Ident::new(&method_name, Span::call_site());
                            quote! {
                                fn #new_method_ident(&mut self, item: #gen_ident ) -> &mut #b_ident {
                                    self.#name.push(item);
                                    self
                                }

                                fn #name(&mut self, #name: #ty) -> &mut #b_ident {
                                    self.#name = #name;
                                    self
                                }
                            }
                        }
                        None => {
                            quote! {
                                fn #name(&mut self, #name: #ty) -> &mut #b_ident {
                                    self.#name = Some(#name);
                                    self
                                }
                            }
                        }
                    }
                }
            }
        }
    });

    let builder_default_fields = fields.iter().map(|f| {
        let name = &f.ident;
        match is_builder_of(f) {
            Some(_) => {
                quote! {
                    #name: Vec::new(),
                }
            }
            None => {
                quote! {
                    #name: None,
                }
            }
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
            None => match is_builder_of(f) {
                None => {
                    return quote! {
                        #name: self.#name.clone().ok_or("sdfsdf")?,
                    }
                }
                Some(_) => {
                    quote! {
                        #name: self.#name.clone(),
                    }
                }
            },
        }
    });

    let t = quote! {
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
        pub fn build(&mut self) -> std::result::Result<#o_name, std::boxed::Box<dyn std::error::Error>> {
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

fn is_builder_of(f: &Field) -> Option<String> {
    for attr in f.attrs.iter() {
        match attr.parse_meta() {
            Ok(meta) => match meta {
                Meta::List(list) => {
                    if list
                        .path
                        .segments
                        .iter()
                        .any(|s| &s.ident.to_string() == "builder")
                    {
                        if list.nested.len() > 0 {
                            match list.nested.first().unwrap() {
                                NestedMeta::Meta(meta) => match meta {
                                    Meta::NameValue(name_value_meta) => {
                                        if name_value_meta
                                            .path
                                            .segments
                                            .iter()
                                            .any(|s| &s.ident.to_string() == "each")
                                        {
                                            match &name_value_meta.lit {
                                                Lit::Str(litstr) => {
                                                    println!("litstr: {:?}", litstr.value());
                                                    return Some(litstr.value());
                                                }
                                                _ => {
                                                    return None;
                                                }
                                            }
                                        }
                                    }
                                    _ => {
                                        return None;
                                    }
                                },
                                _ => {
                                    return None;
                                }
                            }
                        }
                    }
                }
                _ => {
                    return None;
                }
            },
            Err(_) => {
                return None;
            }
        }
    }
    return None;
}

fn get_generic_type_of_vec(f: &Field) -> Option<&Type> {
    match &f.ty {
        Type::Path(type_path) => {
            // TODO: Check if using last() here is correct.
            // let vec_path_segment = type_path.path.segments.last();
            // println!("path segs: {:#?}", type_path.path.segments);
            if type_path.path.segments.len() > 0 {
                match &type_path.path.segments.last().unwrap().arguments {
                    PathArguments::AngleBracketed(gen_args) => {
                        // println!("Gen Args: {:#?}", gen_args);
                        if gen_args.args.len() > 0 {
                            match gen_args.args.last().unwrap() {
                                GenericArgument::Type(ty) => {
                                    return Some(ty);
                                }
                                _ => {
                                    return None;
                                }
                            }
                        } else {
                            return None;
                        }
                    }
                    _ => {
                        println!("not found angleBracketed args");
                        return None;
                    }
                }
            } else {
                return None;
            }
        }
        _ => {
            println!("field is not of type path");
            return None;
        }
    }
}

// TODO: Refactor this method. Damn its soo unreadable! -_-
fn is_field_option_type<'a>(f: &'a syn::Field) -> Option<&'a Type> {
    use syn::{AngleBracketedGenericArguments, Path, TypePath};
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
                            }) => match args.last().unwrap() {
                                // TODO: fix this bug, this code will fail for Option<std::string::String>
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
