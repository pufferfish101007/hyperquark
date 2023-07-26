use proc_macro::TokenStream;
use proc_macro_error::{emit_warning, proc_macro_error};
use quote::{format_ident, quote};
use syn::{parse_macro_input, Data, DeriveInput, Fields, Type};

use std::collections::{HashMap, HashSet};

#[proc_macro_error]
#[proc_macro_derive(EnumFieldGetter)]
pub fn enum_field_getter(stream: TokenStream) -> TokenStream {
    let info = parse_macro_input!(stream as DeriveInput);
    if let Data::Enum(enum_data) = info.data {
        let variants = enum_data.variants.iter();
        let name = info.ident;
        let mut field_info: HashMap<String, (Type, Vec<String>)> = HashMap::new();
        let mut tuple_field_info: HashMap<usize, (Type, Vec<String>)> = HashMap::new();
        let mut incompatible = HashSet::<String>::new();
        let mut tuple_incompatible = HashSet::<usize>::new();
        for variant in variants {
            if let Fields::Named(_) = variant.fields {
                for field in &variant.fields {
                    let ident = field.ident.clone().unwrap().to_string();
                    let field_ty = field.ty.clone();
                    let df = (field_ty.clone(), vec![variant.ident.to_string()]);
                    field_info.entry(ident.clone()).and_modify(|info| {
                        let (ty, used_variants) = info;
                        if quote!{#field_ty}.to_string() != quote!{#ty}.to_string() {
                            emit_warning!(field, "fields must be the same type across all variants - no getter will be emitted for this field");
                            incompatible.insert(ident.clone());
                        } else {
                            used_variants.push(variant.ident.to_string());
                        }
                    }).or_insert(df);
                }
            } else if let Fields::Unnamed(_) = variant.fields {
                for (i, field) in variant.fields.iter().enumerate() {
                    let field_ty = field.ty.clone();
                    let df = (field_ty.clone(), vec![variant.ident.to_string()]);
                    tuple_field_info.entry(i).and_modify(|info| {
                        let (ty, used_variants) = info;
                        if quote!{#field_ty}.to_string() != quote!{#ty}.to_string() {
                            emit_warning!(field, "fields must be the same type across all variants - no getter will be emitted for this field");
                            tuple_incompatible.insert(i);
                        } else {
                            used_variants.push(variant.ident.to_string());
                        }
                    }).or_insert(df);
                }
            }
        }
        for removeable in incompatible {
            field_info.remove(&removeable);
        }
        for tuple_removeable in tuple_incompatible {
            tuple_field_info.remove(&tuple_removeable);
        }
        let getters = field_info.keys().map(|k| format_ident!("{}", k));
        let types = field_info.values().map(|v| v.0.clone());
        let field_info_vec = field_info.iter().collect::<Vec<_>>();
        let matches = field_info_vec.iter().map(|(k, v)| {
            let variants =
                v.1.clone()
                    .iter()
                    .map(|v| format_ident!("{}", v))
                    .collect::<Vec<_>>();
            let field = vec![format_ident!("{}", k); variants.len()];
            quote! {
                match self {
                    #(
                        Self::#variants { #field, .. } => Some(#field),
                    )*
                    _ => None,
                }
            }
        });
        let tuple_getters = tuple_field_info.keys().map(|k| format_ident!("get_{}", k));
        let tuple_types = tuple_field_info.values().map(|v| v.0.clone());
        let tuple_field_info_vec = tuple_field_info.iter().collect::<Vec<_>>();
        let tuple_matches = tuple_field_info_vec.iter().map(|(k, v)| {
            let variants =
                v.1.clone()
                    .iter()
                    .map(|v| format_ident!("{}", v))
                    .collect::<Vec<_>>();
            let preceding = vec![format_ident!("_"); **k];
            let preceding_quote = vec![quote! { #(#preceding,)* }; variants.len()];
            let field = vec![format_ident!("val_{}", k); variants.len()];
            quote! {
                match self {
                    #(
                        Self::#variants(#preceding_quote #field, .. ) => Some(#field),
                    )*
                    _ => None,
                }
            }
        });
        quote! {
            impl #name {
                #(
                pub fn #getters (&self) -> Option<&#types> {
                    #matches
                }
                )*
                #(
                pub fn #tuple_getters (&self) -> Option<&#tuple_types> {
                    #tuple_matches
                }
                )*
            }
        }
        .into()
    } else {
        panic!("macro can only be used on enums");
    }
}
