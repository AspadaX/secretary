/// Macro that generates an object by setting its fields from tuples of field names and values.
///
/// # Arguments
///
/// * `obj` - The initial object to be used as a template for creating a new instance.
/// * `tuples` - A list of tuples where each tuple contains a field name (as an expression) and the content (value) for that field.
///
/// # Example
///
/// ```
/// #[derive(Debug)]
/// struct MyStruct {
///     my_field: String,
/// }
///
/// let updated_obj = generate_from_tuples!(MyStruct { my_field: "initial".to_string() }, [("my_field", "new_value")]);
/// assert_eq!(updated_obj.my_field, "new_value");
/// ```
#[macro_export]
macro_rules! generate_from_tuples {
    ( $obj:expr, [ $( ($field_name:expr, $content:expr) ),* ] ) => {
        {
            let obj = $obj;
            {
                #[allow(unused_mut)]
                let mut new_obj = obj;
                $(
                    new_obj.$field_name = parse_value!($content);
                )*
                new_obj
            }
        }
    };
}

macro_rules! parse_value {
    ($val:expr) => {
        {
            if <T as std::str::FromStr>::from_str(&$val).is_ok() {
                <T as std::str::FromStr>::from_str($val).unwrap()
            } else {
                $val.to_string()
            }
        }
    };
}