use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, parse_macro_input};

/// Derive macro that implements the Task trait for a struct,
/// allowing users to directly use their data structures with LLM generation.
///
/// This macro automatically implements:
/// - The `Task` trait with system prompt generation
/// - The `Default` trait for easy instantiation
/// - A `new()` constructor method
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
///
/// let task = MyData::new();
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

    let field_defaults: Vec<_> = fields
        .iter()
        .map(|field| {
            let field_name: &syn::Ident = field.ident.as_ref().unwrap();
            quote! {
                #field_name: Default::default()
            }
        })
        .collect();

    // Generate field instructions and expansion logic for nested structs
    let field_expansions: Vec<_> = fields
        .iter()
        .map(|field| {
            let field_name: &syn::Ident = field.ident.as_ref().unwrap();
            let field_name_str: String = field_name.to_string();
            let field_type = &field.ty;
            let type_str = quote!(#field_type).to_string();

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

            let combined_instruction = format!("{}: {}", instruction, type_str);

            quote! {
                {
                    // Try to expand nested Task structs
                    let field_value = &self.#field_name;
                    // Check if the field implements Task trait
                    if let Ok(nested_json) = serde_json::to_value(field_value) {
                        match &nested_json {
                            serde_json::Value::Object(obj) => {
                                if !obj.is_empty() {
                                    // For nested objects, try to expand them if they're Task structs
                                    // For now, we'll serialize the nested object and use it as the structure
                                    instruction_map.insert(#field_name_str.to_string(), nested_json);
                                } else {
                                    // Empty object, use instruction string
                                    instruction_map.insert(#field_name_str.to_string(), serde_json::Value::String(#combined_instruction.to_string()));
                                }
                            },
                            _ => {
                                // Not an object, use instruction string
                                instruction_map.insert(#field_name_str.to_string(), serde_json::Value::String(#combined_instruction.to_string()));
                            }
                        }
                    } else {
                        // Serialization failed, use instruction string
                        instruction_map.insert(#field_name_str.to_string(), serde_json::Value::String(#combined_instruction.to_string()));
                    }
                }
            }
        })
        .collect();

    // Also generate the simple field instructions for distributed generation
    let field_instructions: Vec<_> = fields
        .iter()
        .map(|field| {
            let field_name: &syn::Ident = field.ident.as_ref().unwrap();
            let field_name_str: String = field_name.to_string();
            let field_type = &field.ty;
            let type_str = quote!(#field_type).to_string();

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

            let combined_instruction = format!("{}: {}", instruction, type_str);

            quote! {
                (#field_name_str, #combined_instruction)
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

            /// Build instruction JSON structure with nested Task expansion
            fn build_instruction_json(&self) -> serde_json::Value {
                use serde_json::{Value, Map};

                let mut instruction_map = Map::new();

                #(#field_expansions)*

                Value::Object(instruction_map)
            }
        }

        impl ::secretary::traits::Task for #name {
            fn get_system_prompt(&self) -> String {
                use serde_json::{Value, Map};

                // Build the instruction JSON structure directly
                let instruction_json = self.build_instruction_json();

                let mut prompt = String::new();
                prompt.push_str("This is the json structure that you should strictly follow:\n");
                prompt.push_str(&serde_json::to_string_pretty(&instruction_json).unwrap_or_else(|_| "{}".to_string()));

                prompt
            }

            fn get_system_prompts_for_distributed_generation(&self) -> Vec<(String, String)> {
                let mut prompts: Vec<(String, String)> = Vec::new();

                let field_map: std::collections::HashMap<&str, &str> = [
                    #(#field_instructions),*
                ].iter().cloned().collect();

                for (field, instruction) in field_map {
                    let mut prompt = String::new();
                    // Add field-specific instructions
                    prompt.push_str("Output a value according to criteria and wrap them in <result></result>.\n");
                    prompt.push_str(&format!("- {}: {}\n", field, instruction));

                    prompts.push(
                        (field.to_string(), prompt)
                    );
                }

                prompts
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
