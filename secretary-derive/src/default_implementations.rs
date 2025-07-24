use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, Fields, Ident, Type};

use crate::field_types::{detect_task_field_type, TaskFieldType};

pub fn implement_default(
    name: &Ident,
    data: &Data
) -> TokenStream {
    match data {
        Data::Struct(data_struct) => {
            let fields = match &data_struct.fields {
                Fields::Named(fields) => &fields.named,
                _ => {
                    // For non-named fields, return a simple Default implementation
                    return quote! {
                        impl Default for #name {
                            fn default() -> Self {
                                Self::default()
                            }
                        }
                    };
                }
            };

            // Assign default values to each field based on their type
            let field_defaults: Vec<_> = fields
                .iter()
                .map(|field| {
                    let field_name: &syn::Ident = field.ident.as_ref().unwrap();
                    let default_value = generate_default_value(&field.ty);
                    quote! {
                        #field_name: #default_value
                    }
                })
                .collect();

            quote! {
                impl Default for #name {
                    fn default() -> Self {
                        Self {
                            #(#field_defaults),*
                        }
                    }
                }
            }
        }
        _ => {
            // For enums and unions, provide a basic Default implementation
            quote! {
                impl Default for #name {
                    fn default() -> Self {
                        Default::default()
                    }
                }
            }
        }
    }
}

fn generate_default_value(field_type: &Type) -> TokenStream {
    let task_field_type = detect_task_field_type(field_type);
    
    match task_field_type {
        TaskFieldType::VecTask => {
            // Generate a Vec with example data
            if let Type::Path(path) = field_type {
                if let Some(last_segment) = path.path.segments.last() {
                    if let syn::PathArguments::AngleBracketed(args) = &last_segment.arguments {
                        if let Some(syn::GenericArgument::Type(inner_type)) = args.args.first() {
                            let inner_default = generate_default_value(inner_type);
                            return quote! {
                                vec![#inner_default, #inner_default]
                            };
                        }
                    }
                }
            }
            quote! { vec![] }
        }
        TaskFieldType::OptionTask => {
            // Generate Some(example_value)
            if let Type::Path(path) = field_type {
                if let Some(last_segment) = path.path.segments.last() {
                    if let syn::PathArguments::AngleBracketed(args) = &last_segment.arguments {
                        if let Some(syn::GenericArgument::Type(inner_type)) = args.args.first() {
                            let inner_default = generate_default_value(inner_type);
                            return quote! {
                                Some(#inner_default)
                            };
                        }
                    }
                }
            }
            quote! { None }
        }
        TaskFieldType::HashMapTask => {
            // Generate a HashMap with example key-value pairs
            if let Type::Path(path) = field_type {
                if let Some(last_segment) = path.path.segments.last() {
                    if let syn::PathArguments::AngleBracketed(args) = &last_segment.arguments {
                        if let (Some(syn::GenericArgument::Type(key_type)), Some(syn::GenericArgument::Type(value_type))) = 
                            (args.args.first(), args.args.iter().nth(1)) {
                            let key_default = generate_primitive_default(key_type);
                            let value_default = generate_default_value(value_type);
                            return quote! {
                                {
                                    let mut map = std::collections::HashMap::new();
                                    map.insert(#key_default, #value_default);
                                    map.insert(#key_default, #value_default);
                                    map
                                }
                            };
                        }
                    }
                }
            }
            quote! { std::collections::HashMap::new() }
        }
        TaskFieldType::BTreeMapTask => {
            // Generate a BTreeMap with example key-value pairs
            if let Type::Path(path) = field_type {
                if let Some(last_segment) = path.path.segments.last() {
                    if let syn::PathArguments::AngleBracketed(args) = &last_segment.arguments {
                        if let (Some(syn::GenericArgument::Type(key_type)), Some(syn::GenericArgument::Type(value_type))) = 
                            (args.args.first(), args.args.iter().nth(1)) {
                            let key_default = generate_primitive_default(key_type);
                            let value_default = generate_default_value(value_type);
                            return quote! {
                                {
                                    let mut map = std::collections::BTreeMap::new();
                                    map.insert(#key_default, #value_default);
                                    map.insert(#key_default, #value_default);
                                    map
                                }
                            };
                        }
                    }
                }
            }
            quote! { std::collections::BTreeMap::new() }
        }
        TaskFieldType::DirectTask | TaskFieldType::Normal => {
            // For direct Task types and normal types, use Default::default()
            quote! { Default::default() }
        }
    }
}

fn generate_primitive_default(field_type: &Type) -> TokenStream {
    if let Type::Path(path) = field_type {
        if let Some(last_segment) = path.path.segments.last() {
            let type_name = last_segment.ident.to_string();
            match type_name.as_str() {
                "String" => return quote! { "example_key".to_string() },
                "i32" | "i64" | "isize" => return quote! { 1 },
                "u32" | "u64" | "usize" => return quote! { 1 },
                "f32" | "f64" => return quote! { 1.0 },
                "bool" => return quote! { true },
                "char" => return quote! { 'a' },
                _ => {}
            }
        }
    }
    quote! { Default::default() }
}