// use syn::spanned::Spanned;
type StructFields = syn::punctuated::Punctuated<syn::Field, syn::Token![,]>;
pub(crate) fn get_fields_from_derive_input(st: &syn::DeriveInput) -> syn::Result<&StructFields> {
    if let syn::Data::Struct(syn::DataStruct {
        fields: syn::Fields::Named(syn::FieldsNamed { named, .. }),
        ..
    }) = &st.data
    {
        Ok(named)
    } else {
        Err(syn::Error::new_spanned(st, "Must Define on Struct, Not on Enum"))
    }
}

pub(crate) fn get_field_macro_attr_path_value_string(field: &syn::Field, attr_path: &str, allowed_ident_names: Option<Vec<&str>>) -> syn::Result<Option<String>> {
    for attr in &field.attrs {
        if let syn::Meta::NameValue(kv) = &attr.meta {
            if kv.path.is_ident(attr_path) {
                match &kv.value {
                    syn::Expr::Lit(expr) => {
                        if let syn::Lit::Str(ref ident_str) = expr.lit {
                            return Ok(Some(ident_str.value().to_string()));
                        }
                    }
                    _ => (),
                }
            }
            if let Some(ref allowed_ident_names) = allowed_ident_names {
                match kv.path.get_ident() {
                    Some(kv_path_ident) => {
                        let kv_path_name = kv_path_ident.to_string();
                        if allowed_ident_names.iter().find(|allowed_name| *allowed_name == &kv_path_name).is_none() {
                            return Err(syn::Error::new_spanned(field, format!(r#"expected `builder({} = "...")`"#, allowed_ident_names.join("|"))));
                        }
                    }
                    _ => (),
                }
            }
        }
    }
    Ok(None)
}
