use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Data, Fields};

/// Derive macro that implements the Task trait for a struct.
/// This combines the functionality of SystemPrompt and Context traits,
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
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    
    // Extract field information for generating instructions
    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => panic!("Task can only be derived for structs with named fields"),
        },
        _ => panic!("Task can only be derived for structs"),
    };
    
    // Generate field instructions from attributes or field names
    let field_instructions: Vec<_> = fields.iter().map(|field| {
        let field_name: &syn::Ident = field.ident.as_ref().unwrap();
        let field_name_str: String = field_name.to_string();
        
        // Look for #[task(instruction = "...")] attribute
        let instruction: String = field.attrs.iter()
            .find_map(|attr| {
                if attr.path().is_ident("task") {
                    attr.parse_args::<syn::LitStr>().ok().map(|lit| lit.value())
                        .or_else(|| {
                            // Try parsing as instruction = "value"
                            attr.parse_args::<syn::Meta>().ok().and_then(|meta| {
                                if let syn::Meta::NameValue(nv) = meta {
                                    if nv.path.is_ident("instruction") {
                                        if let syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(lit_str), .. }) = &nv.value {
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
    }).collect();
    
    let expanded = quote! {
        impl #name {
            /// Create a new instance with additional instructions
            pub fn new(additional_instructions: Vec<String>) -> Self {
                let mut instance = Self::default();
                instance.additional_instructions = additional_instructions;
                instance.context = ::secretary::message_list::MessageList::new();
                instance
            }
        }
        
        impl ::secretary::traits::Task for #name {
            fn new_with_instructions(additional_instructions: Vec<String>) -> Self {
                Self::new(additional_instructions)
            }
            
            fn get_system_prompt(&self) -> String {
                let mut prompt = String::new();
                prompt.push_str("This is the json structure that you should strictly follow:\n");
                
                // Generate the data model JSON
                let data_model = Self::provide_data_model_instructions();
                prompt.push_str(&::serde_json::to_string(&data_model).unwrap());
                prompt.push_str("\n");
                
                // Add field-specific instructions
                prompt.push_str("Field instructions:\n");
                let field_map: std::collections::HashMap<&str, &str> = [
                    #(#field_instructions),*
                ].iter().cloned().collect();
                
                for (field, instruction) in field_map {
                    prompt.push_str(&format!("- {}: {}\n", field, instruction));
                }
                
                // Add additional instructions
                if !self.get_additional_instructions().is_empty() {
                    prompt.push_str("\nAdditional instructions:\n");
                    for instruction in self.get_additional_instructions() {
                        prompt.push_str(&format!("- {}\n", instruction));
                    }
                }
                
                prompt
            }
            
            fn push(&mut self, role: ::secretary::message_list::Role, content: &str) -> Result<(), ::anyhow::Error> {
                use ::anyhow::anyhow;
                match role {
                    ::secretary::message_list::Role::User => {
                        self.get_context_mut()
                            .push(::secretary::message_list::Message::new(::secretary::message_list::Role::User, content.to_string()));
                    }
                    ::secretary::message_list::Role::Assistant => {
                        self.get_context_mut()
                            .push(::secretary::message_list::Message::new(::secretary::message_list::Role::Assistant, content.to_string()));
                    }
                    ::secretary::message_list::Role::System => {
                        self.get_context_mut()
                            .push(::secretary::message_list::Message::new(::secretary::message_list::Role::System, content.to_string()));
                    }
                }
                Ok(())
            }
            
            fn get_context_mut(&mut self) -> &mut ::secretary::message_list::MessageList {
                &mut self.context
            }
            
            fn get_context(&self) -> ::secretary::message_list::MessageList {
                let mut final_context = ::secretary::message_list::MessageList::new();
                final_context.push(::secretary::message_list::Message::new(
                    ::secretary::message_list::Role::System,
                    self.get_system_prompt(),
                ));
                
                final_context.extend(self.context.clone());
                final_context
            }
            
            fn get_additional_instructions(&self) -> &Vec<String> {
                &self.additional_instructions
            }
            
            fn set_additional_instructions(&mut self, instructions: Vec<String>) {
                self.additional_instructions = instructions;
            }
        }
        
        impl ::secretary::traits::DataModel for #name {
            fn provide_data_model_instructions() -> Self {
                // This will be implemented by the user or use Default if available
                Self::default()
            }
        }
        
        impl ::secretary::traits::ToJSON for #name {}
        impl ::secretary::traits::FromJSON for #name {}
    };
    
    TokenStream::from(expanded)
}