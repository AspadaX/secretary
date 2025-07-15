use syn::Type;

use crate::{errors::ValidationError, utilities::get_field_instruction};

#[derive(Debug, PartialEq, PartialOrd, Ord, Eq)]
pub enum FieldCategory {
    Primitive,
    PotentialTask,
    Unknown,
}

/// Classifies a field type into one of the categories.
/// Can recursively classify nested types.
pub fn classify_field_type(ty: &Type) -> FieldCategory {
    match ty {
        Type::Path(path) => {
            let type_name = path.path.segments.last()
                .map(|seg| seg.ident.to_string())
                .unwrap_or_default();
            
            match type_name.as_str() {
                // Primitive types
                "String" | "i32" | "i64" | "u32" | "u64" | "f32" | "f64" | 
                "bool" | "char" | "isize" | "usize" | "i8" | "i16" | "u8" | "u16" => {
                    FieldCategory::Primitive
                },
                // Standard library types (treated as primitives for Task purposes)
                "Vec" | "Option" | "HashMap" | "BTreeMap" | "HashSet" | "BTreeSet" => {
                    FieldCategory::Primitive
                },
                // Custom types (potential Task implementors)
                _ if !type_name.starts_with("std::") => {
                    FieldCategory::PotentialTask
                },
                _ => FieldCategory::Unknown,
            }
        },
        Type::Reference(reference) => classify_field_type(&reference.elem),
        Type::Array(_) | Type::Slice(_) => FieldCategory::Primitive,
        _ => FieldCategory::Unknown,
    }
}

pub fn validate_field_requirements(fields: &syn::punctuated::Punctuated<syn::Field, syn::token::Comma>) -> Result<(), ValidationError> {
    for field in fields {
        let field_category: FieldCategory = classify_field_type(&field.ty);
        let field_instruction: Option<String> = get_field_instruction(&field);
        
        match field_category {
            FieldCategory::PotentialTask => {
                if field_instruction.is_some() {
                    return Err(ValidationError::TaskFieldWithInstruction(field.ident.as_ref().unwrap().to_string()))
                }
            },
            FieldCategory::Primitive => {
                if field_instruction.is_none() {
                    return Err(ValidationError::PrimitiveMissingInstruction(field.ident.as_ref().unwrap().to_string()))
                }
            },
            FieldCategory::Unknown => {
                return Err(ValidationError::UnknownFieldType(field.ident.as_ref().unwrap().to_string()));
            }
        }
    }
    
    Ok(())
}
