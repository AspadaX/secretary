use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, Field, Fields};

use crate::{
    field_types::{TaskFieldType, detect_task_field_type},
    utilities::{convert_to_json_type, get_instruction},
};

pub struct DataStructureField {
    field: Field,
    name: String,
    instruction: String,
    json_data_type: String,
    task_field_type: TaskFieldType,
}

impl DataStructureField {
    pub fn new(
        field: Field,
        name: String,
        instruction: String,
        json_data_type: String,
        task_field_type: TaskFieldType,
    ) -> Self {
        Self {
            field,
            name,
            instruction,
            json_data_type,
            task_field_type,
        }
    }

    pub fn get_field_prompt(&self) -> String {
        format!(
            "{}: {}, {}\n",
            self.name, self.instruction, self.json_data_type
        )
    }

    pub fn get_task_field_type(&self) -> &TaskFieldType {
        &self.task_field_type
    }

    pub fn get_field_name(&self) -> &str {
        &self.name
    }
}

pub fn get_data_structure_fields(data: &Data) -> Result<Vec<DataStructureField>, TokenStream> {
    match data {
        Data::Struct(content) => {
            let named_fields: syn::punctuated::Punctuated<Field, syn::token::Comma> =
                match &content.fields {
                    Fields::Named(fields) => fields.named.to_owned(),
                    _ => {
                        let error: syn::Error = syn::Error::new_spanned(
                            &content.struct_token,
                            "Unnamed fields are not supported",
                        );
                        return Err(TokenStream::from(error.to_compile_error()));
                    }
                };

            let mut data_structure_fields = Vec::new();

            for field in named_fields.iter() {
                let json_data_type: String = convert_to_json_type(&field.ty);
                let task_field_type: TaskFieldType = detect_task_field_type(&field.ty);

                // Extract task attribute to get instruction (only required for non-DirectTask fields)
                let instruction: String = match task_field_type {
                    TaskFieldType::DirectTask => {
                        // DirectTask fields don't need instruction attributes
                        String::new()
                    }
                    _ => {
                        // All other field types require instruction attributes
                        match get_instruction(&field) {
                            Some(result) => result,
                            None => {
                                let error: syn::Error = syn::Error::new_spanned(
                                    &field,
                                    "Missing required #[task(instruction = \"...\")] attribute",
                                );
                                return Err(TokenStream::from(error.to_compile_error()));
                            }
                        }
                    }
                };

                let name: String = match &field.ident {
                    Some(ident) => ident.to_string(),
                    None => {
                        let error =
                            syn::Error::new_spanned(&field, "Unnamed fields are not supported");
                        return Err(TokenStream::from(error.to_compile_error()));
                    }
                };

                data_structure_fields.push(DataStructureField::new(
                    field.clone(),
                    name,
                    instruction,
                    json_data_type,
                    task_field_type,
                ));
            }

            Ok(data_structure_fields)
        }
        Data::Enum(enum_data) => {
            let error =
                syn::Error::new_spanned(&enum_data.enum_token, "Enums are not supported yet");
            Err(TokenStream::from(error.to_compile_error()))
        }
        Data::Union(union_data) => {
            let error =
                syn::Error::new_spanned(&union_data.union_token, "Unions are not supported");
            Err(TokenStream::from(error.to_compile_error()))
        }
    }
}
