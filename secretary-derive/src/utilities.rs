use syn::{Field, Type};

pub fn convert_to_json_type(rust_type: &Type) -> String {
    match rust_type {
        Type::Array(_) => format!("JSON Array"),
        Type::Slice(_) => format!("JSON Array"),
        Type::Path(path) => {
            if let Some(last_segment) = path.path.segments.last() {
                let type_name = last_segment.ident.to_string();

                match type_name.as_str() {
                    // Primitive types
                    "i32" | "i64" | "isize" => format!("JSON Number"),
                    "u8" | "u16" | "u32" | "u64" | "usize" => format!("JSON Number"),
                    "f32" | "f64" => format!("JSON Number"),
                    "bool" => format!("JSON Boolean"),
                    "String" => format!("JSON String"),

                    // Generic types
                    "Option" => {
                        if let syn::PathArguments::AngleBracketed(args) = &last_segment.arguments {
                            if let Some(syn::GenericArgument::Type(inner_type)) = args.args.first()
                            {
                                let inner_json_type = convert_to_json_type(inner_type);
                                return format!("{} or JSON Null", inner_json_type);
                            }
                        }
                        format!("JSON String or JSON Null")
                    }
                    "Vec" => {
                        if let syn::PathArguments::AngleBracketed(args) = &last_segment.arguments {
                            if let Some(syn::GenericArgument::Type(inner_type)) = args.args.first()
                            {
                                let inner_json_type = convert_to_json_type(inner_type);
                                return format!("{}(s) in a JSON Array", inner_json_type);
                            }
                        }
                        format!("JSON Array")
                    }
                    "HashMap" | "BTreeMap" => format!("JSON Object"),
                    "HashSet" | "BTreeSet" => {
                        if let syn::PathArguments::AngleBracketed(args) = &last_segment.arguments {
                            if let Some(syn::GenericArgument::Type(inner_type)) = args.args.first()
                            {
                                let inner_json_type = convert_to_json_type(inner_type);
                                return format!("JSON Array of {}", inner_json_type.to_lowercase());
                            }
                        }
                        format!("JSON Array")
                    }

                    // Custom types (potential Task implementors)
                    _ => format!("JSON Object"),
                }
            } else {
                format!("JSON Object")
            }
        }
        Type::Reference(reference) => convert_to_json_type(&reference.elem),
        Type::Tuple(_) => {
            format!("JSON Array") // Rust tuples map to JSON arrays
        }
        _ => format!("JSON Null"), // Default case for unknown types
    }
}

/// Get the field instruction annotated with #[task(instruction = "...")] attribute
/// in each field of a struct
pub fn get_field_instruction(field: &Field) -> Option<String> {
    let mut field_instruction: Option<String> = None;

    for field_attr in field.attrs.iter() {
        if field_attr.path().is_ident("task") {
            field_instruction = field_attr
                .parse_args::<syn::LitStr>()
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
