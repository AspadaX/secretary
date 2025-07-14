use syn::{Field, Type};

pub fn convert_to_json_type(rust_type: &Type) -> String {
    match rust_type {
        Type::Array(_) => format!("JSON Array"),
        Type::Slice(_) => format!("JSON Array"),
        Type::Path(path) => {
            let path_str = path.path.segments.iter()
                .map(|s| s.ident.to_string())
                .collect::<Vec<_>>()
                .join("::");
            match path_str.as_str() {
                "i32" | "i64" | "isize" => format!("JSON Number"),
                "u8" | "u16" | "u32" | "u64" | "usize" => format!("JSON Number"),
                "f32" | "f64" => format!("JSON Number"),
                "bool" => format!("JSON Boolean"),
                "String" => format!("JSON String"),
                _ => format!("JSON Object"), // Default to object for custom types
            }
        },
        Type::Reference(reference) => convert_to_json_type(&reference.elem),
        Type::Tuple(_) => {
            format!("JSON Array") // Rust tuples map to JSON arrays
        },
        _ => format!("JSON Null"), // Default case for unknown types
    }
}

/// Get the field instruction annotated with #[task(instruction = "...")] attribute
/// in each field of a struct
pub fn get_field_instruction(field: &Field) -> Option<String> {
    let mut field_instruction: Option<String> = None;
    
    for field_attr in field.attrs.iter() {
        if field_attr.path().is_ident("task") {
            field_instruction = field_attr.parse_args::<syn::LitStr>()
                .ok()
                .map(|lit| lit.value())
                .or_else(|| {
                    // Try parsing as instruction = "value"
                    field_attr.parse_args::<syn::Meta>().ok().and_then(|meta| {
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
                });
        }
    }
    
    field_instruction
}
