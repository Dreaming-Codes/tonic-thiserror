extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Attribute, Meta, NestedMeta, Data};

#[proc_macro_derive(TonicThisError, attributes(code))]
pub fn tonic_thiserror_derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    impl_tonic_thiserror(&ast).unwrap_or_else(|e| e.to_compile_error()).into()
}

fn impl_tonic_thiserror(ast: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let name = &ast.ident;
    let variants = match &ast.data {
        Data::Enum(data) => &data.variants,
        _ => return Err(syn::Error::new_spanned(&ast, "TonicThisError can only be used on enums")),
    };

    let mut arms = proc_macro2::TokenStream::new();

    for variant in variants {
        let ident = &variant.ident;
        let code_attr = variant
            .attrs
            .iter()
            .find(|attr| attr.path.is_ident("code"))
            .ok_or_else(|| syn::Error::new_spanned(variant, "Missing `code` attribute"))?;

        let code = parse_code_attr(code_attr)?;

        let arm = quote! {
            #name::#ident { .. } => ::tonic::Status::new(#code, format!("{}", err)),
        };

        arms.extend(arm);
    }

    let gen = quote! {
        impl From<#name> for ::tonic::Status {
            fn from(err: #name) -> ::tonic::Status {
                match err {
                    #arms
                }
            }
        }
    };

    Ok(gen)
}

fn parse_code_attr(attr: &Attribute) -> syn::Result<proc_macro2::TokenStream> {
    let meta = attr.parse_meta()?;
    let nested = match meta {
        Meta::List(meta) => {
            if meta.nested.len() != 1 {
                return Err(syn::Error::new_spanned(attr, "Expected exactly one `code`"));
            }
            meta.nested.first().unwrap().clone()
        }
        _ => return Err(syn::Error::new_spanned(attr, "Expected `code` attribute to be a list")),
    };

    match nested {
        NestedMeta::Meta(Meta::Path(path)) => {
            // Assuming the code is one of tonic::Code variants
            // No need to parse a string; it's directly the variant name
            if path.segments.len() == 1 {
                let segment = &path.segments[0];
                let ident = &segment.ident;
                Ok(quote! { ::tonic::Code::#ident })
            } else {
                Err(syn::Error::new_spanned(path, "Expected tonic::Code variant"))
            }
        }
        _ => Err(syn::Error::new_spanned(attr, "Expected tonic::Code variant for `code` attribute")),
    }
}
