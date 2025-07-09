use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, parse_macro_input};

/// Derive macro that implements the Task trait for a struct.
/// allowing users to directly use their data structures with LLM generation.
///
/// # Example
///
/// ```rust
/// use secretary::Task;
/// use serde::{Serialize, Deserialize};
///
/// #[derive(Task, Serialize, Deserialize, Debug)]
/// struct MyData {
///     #[task(instruction = "Extract the name from the input")]
///     name: String,
///     #[task(instruction = "Extract the age as a number")]
///     age: u32,
/// }
/// ```
#[proc_macro_derive(Task, attributes(task))]
pub fn derive_task(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input as DeriveInput);
    let name: &syn::Ident = &input.ident;

    // Extract field information for generating instructions
    let fields: &syn::punctuated::Punctuated<syn::Field, syn::token::Comma> = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => panic!("Task can only be derived for structs with named fields"),
        },
        _ => panic!("Task can only be derived for structs"),
    };
    
    let field_defaults: Vec<_> = fields.iter()
        .map(|field| {
            let field_name: &syn::Ident = field.ident.as_ref().unwrap();
            quote!{
                #field_name: Default::default()
            }
        })
        .collect();

    // Generate field instructions from attributes or field names
    let field_instructions: Vec<_> = fields
        .iter()
        .map(|field| {
            let field_name: &syn::Ident = field.ident.as_ref().unwrap();
            let field_name_str: String = field_name.to_string();

            // Look for #[task(instruction = "...")] attribute
            let instruction: String = field
                .attrs
                .iter()
                .find_map(|attr| {
                    if attr.path().is_ident("task") {
                        attr.parse_args::<syn::LitStr>()
                            .ok()
                            .map(|lit| lit.value())
                            .or_else(|| {
                                // Try parsing as instruction = "value"
                                attr.parse_args::<syn::Meta>().ok().and_then(|meta| {
                                    if let syn::Meta::NameValue(nv) = meta {
                                        if nv.path.is_ident("instruction") {
                                            if let syn::Expr::Lit(syn::ExprLit {
                                                lit: syn::Lit::Str(lit_str),
                                                ..
                                            }) = &nv.value
                                            {
                                                return Some(lit_str.value());
                                            }
                                        }
                                    }
                                    None
                                })
                            })
                    } else {
                        None
                    }
                })
                .unwrap_or_else(|| format!("Extract the {} field from the input", field_name_str));

            quote! {
                (#field_name_str, #instruction)
            }
        })
        .collect();

    let expanded: proc_macro2::TokenStream = quote! {
        impl #name {
            /// Create a new instance with additional instructions
            pub fn new() -> Self {
                let mut instance = Self::default();
                instance
            }
        }

        impl ::secretary::traits::Task for #name {
            fn get_system_prompt(&self) -> String {
                let mut prompt = String::new();
                prompt.push_str("This is the json structure that you should strictly follow:\n");

                // Add field-specific instructions
                prompt.push_str("Field instructions:\n");
                let field_map: std::collections::HashMap<&str, &str> = [
                    #(#field_instructions),*
                ].iter().cloned().collect();

                for (field, instruction) in field_map {
                    prompt.push_str(&format!("- {}: {}\n", field, instruction));
                }

                prompt
            }
        }
        
        impl Default for #name {
            fn default() -> Self {
                Self {
                    #(#field_defaults),*
                }
            }
        }
    };

    TokenStream::from(expanded)
}
