use quote::quote;
use syn::Ident;

use crate::{data_structure::DataStructureField, field_types::TaskFieldType};

pub fn implement_task_trait(
    name: &Ident,
    data_structure_fields: Vec<DataStructureField>,
) -> proc_macro2::TokenStream {
    let field_implementations: Vec<proc_macro2::TokenStream> = implement_get_system_prompt(&data_structure_fields);
    let distributed_field_processing: Vec<proc_macro2::TokenStream> = implement_field_processing_code(&data_structure_fields);

    quote! {
        impl Task for #name {
            fn get_system_prompt(&self) -> String {
                let mut prompt = String::new();
                #(#field_implementations)*
                
                prompt.push_str(&serde_json::to_string_pretty(&self).unwrap());
                
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

pub fn implement_new_method(name: &Ident) -> proc_macro2::TokenStream {
    quote! {
        impl #name {
            pub fn new() -> Self {
                Self::default()
            }
        }
    }
}

fn implement_get_system_prompt(data_structure_fields: &Vec<DataStructureField>) -> Vec<proc_macro2::TokenStream> {
    data_structure_fields
        .iter()
        .map(|field| {
            let field_name_ident =
                syn::Ident::new(field.get_field_name(), proc_macro2::Span::call_site());
            let field_prompt = field.get_field_prompt();
            let field_name = field.get_field_name();

            match field.get_task_field_type() {
                TaskFieldType::Normal => {
                    quote! {
                        prompt.push_str(#field_prompt);
                    }
                }
                TaskFieldType::DirectTask => {
                    quote! {
                        prompt.push_str(&format!("\n--- {} Task Details ---\n", #field_name));
                        prompt.push_str(&self.#field_name_ident.get_system_prompt());
                        prompt.push_str(&format!("--- End of {} Task ---\n\n", #field_name));
                    }
                }
                TaskFieldType::VecTask => {
                    quote! {
                        prompt.push_str(#field_prompt);
                        if !self.#field_name_ident.is_empty() {
                            prompt.push_str(&format!("\n--- {} Collection (any number of items) ---\n", #field_name));
                            for (index, item) in self.#field_name_ident.iter().enumerate() {
                                prompt.push_str(&item.get_system_prompt());
                                prompt.push('\n');
                            }
                            prompt.push_str(&format!("--- End of {} Collection ---\n\n", #field_name));
                        } else {
                            prompt.push_str(&format!(" (Collection is empty)\n"));
                        }
                    }
                }
                TaskFieldType::OptionTask => {
                    quote! {
                        prompt.push_str(#field_prompt);
                        if let Some(ref item) = self.#field_name_ident {
                            prompt.push_str(&format!("\n--- {} Optional Task (Present) ---\n", #field_name));
                            prompt.push_str(&item.get_system_prompt());
                            prompt.push_str(&format!("--- End of {} Optional Task ---\n\n", #field_name));
                        } else {
                            prompt.push_str(&format!(" (Optional field is None)\n"));
                        }
                    }
                }
                TaskFieldType::HashMapTask | TaskFieldType::BTreeMapTask => {
                    let collection_type = if matches!(field.get_task_field_type(), TaskFieldType::HashMapTask) { "HashMap" } else { "BTreeMap" };
                    quote! {
                        prompt.push_str(#field_prompt);
                        if !self.#field_name_ident.is_empty() {
                            prompt.push_str(&format!("\n--- {} {} ({} entries) ---\n", #field_name, #collection_type, self.#field_name_ident.len()));
                            for (key, value) in &self.#field_name_ident {
                                prompt.push_str(&format!("  Key '{}': ", key));
                                prompt.push_str(&value.get_system_prompt());
                                prompt.push('\n');
                            }
                            prompt.push_str(&format!("--- End of {} {} ---\n\n", #field_name, #collection_type));
                        } else {
                            prompt.push_str(&format!(" ({} is empty)\n", #collection_type));
                        }
                    }
                }
            }
        })
        .collect()
}

pub fn implement_field_processing_code(
    data_structure_fields: &Vec<DataStructureField>,
) -> Vec<proc_macro2::TokenStream> {
    data_structure_fields
        .iter()
        .map(|field| {
            let field_name_ident = syn::Ident::new(field.get_field_name(), proc_macro2::Span::call_site());
            let field_name_str = field.get_field_name();
            let field_task_type = field.get_task_field_type();
            
            match field_task_type {
                TaskFieldType::Normal => {
                    // Handle primitive fields with their instructions
                    let field_prompt = field.get_field_prompt();
                    
                    quote! {
                        {
                            let field_path = if prefix.is_empty() {
                                #field_name_str.to_string()
                            } else {
                                format!("{}.{}", prefix, #field_name_str)
                            };
                            
                            let mut prompt = String::new();
                            prompt.push_str("Output a value according to criteria and wrap them in <result></result>.\n");
                            prompt.push_str(&format!("- {}\n", #field_prompt));
                            prompts.push((field_path, prompt));
                        }
                    }
                },
                TaskFieldType::DirectTask => {
                    // Handle Task struct fields by delegating to their implementation
                    quote! {
                        {
                            let field_path = if prefix.is_empty() {
                                #field_name_str.to_string()
                            } else {
                                format!("{}.{}", prefix, #field_name_str)
                            };
                            
                            // Recursively call the nested Task's distributed generation
                            let nested_prompts = self.#field_name_ident.get_system_prompts_for_distributed_generation();
                            
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
                TaskFieldType::VecTask => {
                    // Handle Vec<Task> fields
                    quote! {
                        {
                            let field_path = if prefix.is_empty() {
                                #field_name_str.to_string()
                            } else {
                                format!("{}.{}", prefix, #field_name_str)
                            };
                            
                            for (index, item) in self.#field_name_ident.iter().enumerate() {
                                let item_path = format!("{}[{}]", field_path, index);
                                let nested_prompts = item.get_system_prompts_for_distributed_generation();
                                
                                for (nested_path, nested_prompt) in nested_prompts {
                                    let full_path = if nested_path.is_empty() {
                                        item_path.clone()
                                    } else {
                                        format!("{}.{}", item_path, nested_path)
                                    };
                                    prompts.push((full_path, nested_prompt));
                                }
                            }
                        }
                    }
                },
                TaskFieldType::OptionTask => {
                    // Handle Option<Task> fields
                    quote! {
                        {
                            let field_path = if prefix.is_empty() {
                                #field_name_str.to_string()
                            } else {
                                format!("{}.{}", prefix, #field_name_str)
                            };
                            
                            if let Some(ref item) = self.#field_name_ident {
                                let nested_prompts = item.get_system_prompts_for_distributed_generation();
                                
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
                    }
                },
                TaskFieldType::HashMapTask | TaskFieldType::BTreeMapTask => {
                    // Handle HashMap<K, Task> and BTreeMap<K, Task> fields
                    quote! {
                        {
                            let field_path = if prefix.is_empty() {
                                #field_name_str.to_string()
                            } else {
                                format!("{}.{}", prefix, #field_name_str)
                            };
                            
                            for (key, value) in &self.#field_name_ident {
                                let item_path = format!("{}[{}]", field_path, key);
                                let nested_prompts = value.get_system_prompts_for_distributed_generation();
                                
                                for (nested_path, nested_prompt) in nested_prompts {
                                    let full_path = if nested_path.is_empty() {
                                        item_path.clone()
                                    } else {
                                        format!("{}.{}", item_path, nested_path)
                                    };
                                    prompts.push((full_path, nested_prompt));
                                }
                            }
                        }
                    }
                },
            }
        })
        .collect()
}
