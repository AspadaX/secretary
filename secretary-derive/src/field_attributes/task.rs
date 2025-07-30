use syn::{Ident, LitStr, Token, parse::Parse};

pub struct TaskFieldAttributes {
    pub instruction: Option<String>,
}

impl Parse for TaskFieldAttributes {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut instruction: Option<String> = None;

        while !input.is_empty() {
            let name: Ident = input.parse()?;
            input.parse::<Token![=]>()?;
            let value: LitStr = input.parse()?;

            match name.to_string().as_str() {
                "instruction" => instruction = Some(value.value()),
                _ => return Err(syn::Error::new(name.span(), "Unknown attribute parameter")),
            }

            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(TaskFieldAttributes { instruction })
    }
}
