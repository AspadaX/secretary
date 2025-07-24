use syn::Type;

#[derive(Debug, PartialEq, PartialOrd, Ord, Eq)]
pub enum FieldCategory {
    Primitive,
    PotentialTask,
    Unknown,
}

#[derive(Debug, PartialEq, Clone)]
pub enum TaskFieldType {
    Normal,                    // Regular field, no Task
    DirectTask,               // field: SomeTaskType  
    VecTask,                  // field: Vec<SomeTaskType>
    OptionTask,               // field: Option<SomeTaskType>
    HashMapTask,              // field: HashMap<K, SomeTaskType>
    BTreeMapTask,             // field: BTreeMap<K, SomeTaskType>
}

/// Classifies a field type into one of the categories.
/// Can recursively classify nested types.
pub fn classify_field_type(ty: &Type) -> FieldCategory {
    match ty {
        Type::Path(path) => {
            let type_name = path
                .path
                .segments
                .last()
                .map(|seg| seg.ident.to_string())
                .unwrap_or_default();

            match type_name.as_str() {
                // Primitive types
                "String" | "i32" | "i64" | "u32" | "u64" | "f32" | "f64" | "bool" | "char"
                | "isize" | "usize" | "i8" | "i16" | "u8" | "u16" => FieldCategory::Primitive,
                // Standard library types (treated as primitives for Task purposes)
                "Vec" | "Option" | "HashMap" | "BTreeMap" | "HashSet" | "BTreeSet" => {
                    FieldCategory::Primitive
                }
                // Custom types (potential Task implementors)
                _ if !type_name.starts_with("std::") => FieldCategory::PotentialTask,
                _ => FieldCategory::Unknown,
            }
        }
        Type::Reference(reference) => classify_field_type(&reference.elem),
        Type::Array(_) | Type::Slice(_) => FieldCategory::Primitive,
        _ => FieldCategory::Unknown,
    }
}

/// Detects if a field type contains Task implementations and what kind of container it is
pub fn detect_task_field_type(ty: &Type) -> TaskFieldType {
    match ty {
        Type::Path(path) => {
            if let Some(last_segment) = path.path.segments.last() {
                let type_name = last_segment.ident.to_string();
                
                match type_name.as_str() {
                    "Vec" => {
                        if let syn::PathArguments::AngleBracketed(args) = &last_segment.arguments {
                            if let Some(syn::GenericArgument::Type(inner_type)) = args.args.first() {
                                if classify_field_type(inner_type) == FieldCategory::PotentialTask {
                                    return TaskFieldType::VecTask;
                                }
                            }
                        }
                        TaskFieldType::Normal
                    }
                    "Option" => {
                        if let syn::PathArguments::AngleBracketed(args) = &last_segment.arguments {
                            if let Some(syn::GenericArgument::Type(inner_type)) = args.args.first() {
                                if classify_field_type(inner_type) == FieldCategory::PotentialTask {
                                    return TaskFieldType::OptionTask;
                                }
                            }
                        }
                        TaskFieldType::Normal
                    }
                    "HashMap" => {
                        if let syn::PathArguments::AngleBracketed(args) = &last_segment.arguments {
                            // For HashMap<K, V>, we check the second argument (value type)
                            if let Some(syn::GenericArgument::Type(value_type)) = args.args.iter().nth(1) {
                                if classify_field_type(value_type) == FieldCategory::PotentialTask {
                                    return TaskFieldType::HashMapTask;
                                }
                            }
                        }
                        TaskFieldType::Normal
                    }
                    "BTreeMap" => {
                        if let syn::PathArguments::AngleBracketed(args) = &last_segment.arguments {
                            // For BTreeMap<K, V>, we check the second argument (value type)
                            if let Some(syn::GenericArgument::Type(value_type)) = args.args.iter().nth(1) {
                                if classify_field_type(value_type) == FieldCategory::PotentialTask {
                                    return TaskFieldType::BTreeMapTask;
                                }
                            }
                        }
                        TaskFieldType::Normal
                    }
                    // Custom types (potential direct Task implementors)
                    _ if !type_name.starts_with("std::") && classify_field_type(ty) == FieldCategory::PotentialTask => {
                        TaskFieldType::DirectTask
                    }
                    _ => TaskFieldType::Normal,
                }
            } else {
                TaskFieldType::Normal
            }
        }
        Type::Reference(reference) => detect_task_field_type(&reference.elem),
        _ => TaskFieldType::Normal,
    }
}
