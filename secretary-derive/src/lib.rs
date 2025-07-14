mod utilities;
mod implementations;
mod errors;
mod field;

use errors::ValidationError;
use field::{classify_field_type, validate_field_requirements, FieldCategory};
use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, parse_macro_input};

use utilities::{convert_to_json_type, get_field_instruction};
use implementations::{implement_default, implement_task};

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
    let mut expanded: proc_macro2::TokenStream = proc_macro2::TokenStream::new();

    // Extract field information for generating instructions
    let fields: &syn::punctuated::Punctuated<syn::Field, syn::token::Comma> = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => panic!("Task can only be derived for structs with named fields"),
        },
        _ => panic!("Task can only be derived for structs"),
    };
    
    // Validate
    if let Err(validation_error) = validate_field_requirements(fields) {
        match validation_error {
            ValidationError::PrimitiveMissingInstruction(field_name) => {
                panic!(
                    "Field '{}' is a primitive type and must have a #[task(instruction = \"...\")] attribute",
                    field_name
                );
            },
            ValidationError::TaskFieldWithInstruction(field_name) => {
                panic!(
                    "Field '{}' appears to be a Task struct and should NOT have an instruction attribute. \
                        Task structs define their own instructions internally.",
                    field_name
                );
            },
            ValidationError::UnknownFieldType(field_name) => {
                panic!(
                    "Field '{}' has an unknown type. Only primitive types with instructions \
                        or custom structs implementing Task are allowed.",
                    field_name
                );
            },
        }
    }
    
    // Add `where` clause to fields with Task impl
    let task_field_types: Vec<_> = fields.iter()
        .filter(|field| classify_field_type(&field.ty) == FieldCategory::PotentialTask)
        .map(|field| &field.ty)
        .collect();
    let trait_bounds: proc_macro2::TokenStream = if !task_field_types.is_empty() {
        quote! {
            where 
                #(#task_field_types: ::secretary::traits::Task,)*
        }
    } else {
        quote! {}
    };

    // Generate field instructions and expansion logic for nested structs
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

    // Generate field processing code for distributed generation
    todo!();
    let distributed_field_processing: Vec<_> = fields
        .iter()
        .map(|field| {
            let field_name: &syn::Ident = field.ident.as_ref().unwrap();
            let field_name_str: String = field_name.to_string();
            let field_type: String = convert_to_json_type(&field.ty);

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
                {
                    let field_name = #field_name_str;
                    let field_path = if prefix.is_empty() {
                        field_name.to_string()
                    } else {
                        format!("{}.{}", prefix, field_name)
                    };

                    let field_value = &self.#field_name;
                    let field_type = #field_type;
                    let instruction = #instruction;
                    let combined_instruction = format!("{}: {}", instruction, field_type);
                    
                    // Try to check if this field implements Task trait by attempting to call collect_distributed_prompts
                    // This is a compile-time check - if the field implements Task, this will compile
                    if let Ok(_) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        // Try to serialize to see if it's a complex object
                        serde_json::to_value(field_value)
                    })) {
                        if let Ok(nested_json) = serde_json::to_value(field_value) {
                            match nested_json {
                                serde_json::Value::Object(obj) if !obj.is_empty() => {
                                    // This might be a nested Task struct, try to get its distributed prompts
                                    // For now, we'll check if we can call the method on it
                                    
                                    // If it's a complex object, treat each sub-field as a separate prompt
                                    for (sub_field, _) in obj {
                                        let sub_field_path = format!("{}.{}", field_path, sub_field);
                                        let mut prompt = String::new();
                                        prompt.push_str("Output a value according to criteria and wrap them in <result></result>.\n");
                                        prompt.push_str(&format!("- {}: {}\n", sub_field, combined_instruction));
                                        prompts.push((sub_field_path, prompt));
                                    }
                                },
                                _ => {
                                    // Simple field, add its instruction
                                    let mut prompt = String::new();
                                    prompt.push_str("Output a value according to criteria and wrap them in <result></result>.\n");
                                    prompt.push_str(&format!("- {}: {}\n", field_path, combined_instruction));
                                    prompts.push((field_path, prompt));
                                }
                            }
                        } else {
                            // Serialization failed, treat as simple field
                            let mut prompt = String::new();
                            prompt.push_str("Output a value according to criteria and wrap them in <result></result>.\n");
                            prompt.push_str(&format!("- {}: {}\n", field_path, combined_instruction));
                            prompts.push((field_path, prompt));
                        }
                    } else {
                        // Panic occurred, treat as simple field
                        let mut prompt = String::new();
                        prompt.push_str("Output a value according to criteria and wrap them in <result></result>.\n");
                        prompt.push_str(&format!("- {}: {}\n", field_path, combined_instruction));
                        prompts.push((field_path, prompt));
                    }
                }
            }
        })
        .collect();

    expanded.extend(implement_default(&name, &fields));
    expanded.extend(implement_task(&name, &trait_bounds, &distributed_field_processing));
    expanded.extend(quote! {
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
    });

    TokenStream::from(expanded)
}
