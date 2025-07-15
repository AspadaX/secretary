mod errors;
mod field;
mod implementations;
mod utilities;

use errors::ValidationError;
use field::{FieldCategory, classify_field_type, validate_field_requirements};
use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, parse_macro_input};

use implementations::{
    implement_build_instruction_json, implement_default, implement_field_processing_code,
    implement_task,
};

/// A derive macro that implements the `Task` trait for a struct, enabling seamless integration
/// with LLM-based data extraction workflows.
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
///
/// ## Field-Level Instructions
///
/// Use `#[task(instruction = "...")]` to guide the LLM's extraction for each field.
///
/// ## System Prompt Generation
///
/// The macro generates a system prompt that instructs the LLM to return a JSON object
/// matching the struct's schema. The field-level instructions are embedded within this
/// prompt to ensure accurate extraction.
///
/// ## Error Handling
///
/// - **Compile-Time**: If a field is missing an instruction or there's a configuration error, the macro
///   will produce a compile-time error.
/// - **Run-Time**: If the LLM returns data that cannot be deserialized into the specified field types,
///   a `SecretaryError::FieldDeserializationError` will be returned at runtime, detailing the specific fields that failed.
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
            }
            ValidationError::TaskFieldWithInstruction(field_name) => {
                panic!(
                    "Field '{}' appears to be a Task struct and should NOT have an instruction attribute. \
                        Task structs define their own instructions internally.",
                    field_name
                );
            }
            ValidationError::UnknownFieldType(field_name) => {
                panic!(
                    "Field '{}' has an unknown type. Only primitive types with instructions \
                        or custom structs implementing Task are allowed.",
                    field_name
                );
            }
        }
    }

    // Add `where` clause to fields with Task impl
    let task_field_types: Vec<_> = fields
        .iter()
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

    // Generate field instructions and expansion logic for normal json generation
    let field_expansions: Vec<proc_macro2::TokenStream> = implement_build_instruction_json(fields);

    // Generate field processing code for distributed generation
    let distributed_field_processing: Vec<proc_macro2::TokenStream> =
        implement_field_processing_code(fields);

    expanded.extend(implement_default(&name, &fields));
    expanded.extend(implement_task(
        &name,
        &trait_bounds,
        &distributed_field_processing,
    ));
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
