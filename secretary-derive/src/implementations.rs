use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

use crate::{field::{classify_field_type, FieldCategory}, utilities::{convert_to_json_type, get_field_instruction}};

pub fn implement_default(name: &Ident, fields: &syn::punctuated::Punctuated<syn::Field, syn::token::Comma>) -> TokenStream {
    // Assign default values to each field
    let field_defaults: Vec<_> = fields
        .iter()
        .map(|field| {
            let field_name: &syn::Ident = field.ident.as_ref().unwrap();
            quote! {
                #field_name: Default::default()
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

pub fn implement_task(name: &Ident, trait_bounds: &proc_macro2::TokenStream, distributed_field_processing: &Vec<proc_macro2::TokenStream>) -> TokenStream {
    quote! {
        impl ::secretary::traits::Task for #name 
        #trait_bounds
        {
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
                let prefix = String::new();
    
                #(#distributed_field_processing)*
    
                prompts
            }
        }
    }
}

pub fn implement_build_instruction_json(fields: &syn::punctuated::Punctuated<syn::Field, syn::token::Comma>) -> Vec<proc_macro2::TokenStream> {
    let field_expansions: Vec<_> = fields
        .iter()
        .map(|field| {
            let field_name: &syn::Ident = field.ident.as_ref().unwrap();
            let field_name_str: String = field_name.to_string();
            let field_type: &syn::Type = &field.ty;
            let type_str: String = convert_to_json_type(field_type);

            // Look for #[task(instruction = "...")] attribute
            // Nested structures should not have an instruction
            let instruction: String = get_field_instruction(field)
                .unwrap_or_else(|| format!("Extract the {} field from the input", field_name_str));

            let combined_instruction: String = format!("{}: {}", instruction, type_str);

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
    field_expansions
}

pub fn implement_field_processing_code(fields: &syn::punctuated::Punctuated<syn::Field, syn::token::Comma>) -> Vec<proc_macro2::TokenStream> {
    fields
        .iter()
        .map(|field| {
            let field_name = field.ident.as_ref().unwrap();
            let field_name_str = field_name.to_string();
            let field_category = classify_field_type(&field.ty);
            
            match field_category {
                FieldCategory::Primitive => {
                    // Handle primitive fields with their instructions
                    let instruction = get_field_instruction(field)
                        .expect("Primitive field must have instruction (validated earlier)");
                    let field_type_str = convert_to_json_type(&field.ty);
                    let combined_instruction = format!("{}: {}", instruction, field_type_str);
                    
                    quote! {
                        {
                            let field_path = if prefix.is_empty() {
                                #field_name_str.to_string()
                            } else {
                                format!("{}.{}", prefix, #field_name_str)
                            };
                            
                            let mut prompt = String::new();
                            prompt.push_str("Output a value according to criteria and wrap them in <result></result>.\n");
                            prompt.push_str(&format!("- {}: {}\n", field_path, #combined_instruction));
                            prompts.push((field_path, prompt));
                        }
                    }
                },
                FieldCategory::PotentialTask => {
                    // Handle Task struct fields by delegating to their implementation
                    quote! {
                        {
                            let field_path = if prefix.is_empty() {
                                #field_name_str.to_string()
                            } else {
                                format!("{}.{}", prefix, #field_name_str)
                            };
                            
                            // Recursively call the nested Task's distributed generation
                            // This will only compile if the field implements Task (enforced by trait bounds)
                            let nested_prompts = self.#field_name.get_system_prompts_for_distributed_generation();
                            
                            for (nested_path, nested_prompt) in nested_prompts {
                                let full_path = if nested_path.is_empty() {
                                    field_path.clone()
                                } else {
                                    format!("{}.{}", field_path, nested_path)
                                };
                                prompts.push((full_path, nested_prompt));
                            }
                        }
                    }
                },
                FieldCategory::Unknown => {
                    // This should be caught by validation, but add fallback
                    panic!("Unknown field type for field: {}", field_name_str);
                },
            }
        })
        .collect()
}
