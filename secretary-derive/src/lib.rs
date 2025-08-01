mod data_structure_field;
mod default_implementations;
mod field_attributes;
mod field_types;
mod task_implementations;
mod utilities;

use default_implementations::implement_default;
use proc_macro::TokenStream;
use syn::{DeriveInput, parse_macro_input};

use data_structure_field::{DataStructureField, get_data_structure_fields};
use task_implementations::{implement_new_method, implement_task_trait};

#[proc_macro_derive(Task, attributes(task))]
pub fn derive_task(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input as DeriveInput);
    let name: &syn::Ident = &input.ident;
    let mut expanded: proc_macro2::TokenStream = proc_macro2::TokenStream::new();

    let data_structure_fields: Vec<DataStructureField> =
        match get_data_structure_fields(&input.data) {
            Ok(fields) => fields,
            Err(error) => {
                return error;
            }
        };

    let default_impl = implement_default(name, &input.data);
    let task_impl = implement_task_trait(name, data_structure_fields);
    let new_impl = implement_new_method(name);

    expanded.extend(default_impl);
    expanded.extend(task_impl);
    expanded.extend(new_impl);

    TokenStream::from(expanded)
}
