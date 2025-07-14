use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

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