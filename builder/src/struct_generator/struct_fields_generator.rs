use std::vec;

use syn::{parse::Parser, spanned::Spanned};

type StructFields = syn::punctuated::Punctuated<syn::Field, syn::Token![,]>;
fn get_fields_from_derive_input(st: &syn::DeriveInput) -> syn::Result<&StructFields> {
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

fn get_generic_inner_type<'a>(r#type: &'a syn::Type, outer_ident_name: &str) -> Option<&'a syn::Type> {
    if let syn::Type::Path(syn::TypePath { path: syn::Path { segments, .. }, .. }) = r#type {
        if let Some(seg) = segments.last() {
            if seg.ident.to_string() == outer_ident_name {
                if let syn::PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments { args, .. }) = &seg.arguments {
                    if let Some(syn::GenericArgument::Type(inner_type)) = args.first() {
                        return Some(inner_type);
                    }
                }
            }
        }
    }
    None
}

fn get_field_macro_attr_path_value(field: &syn::Field, attr_path: &str) -> Option<syn::Ident> {
    for attr in &field.attrs {
        if let syn::Meta::List(list) = &attr.meta {
            if let Some(p) = list.path.segments.first() {
                if p.ident == "builder" {
                    let nested_metas = syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated.parse2(list.tokens.clone()).unwrap();
                    for nested_meta in nested_metas.iter() {
                        if let syn::Meta::NameValue(kv) = nested_meta {
                            if kv.path.is_ident(attr_path) {
                                match &kv.value {
                                    syn::Expr::Lit(expr) => {
                                        if let syn::Lit::Str(ref ident_str) = expr.lit {
                                            return Some(syn::Ident::new(ident_str.value().as_str(), field.span()));
                                        }
                                    }
                                    _ => (),
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

pub(crate) fn generate(st: &syn::DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let fields = get_fields_from_derive_input(st)?;

    let idents: Vec<_> = fields.iter().map(|field| &field.ident).collect();
    let mut types = vec![];
    for field in fields.iter() {
        let r#type = &field.ty;
        if let Some(inner_type) = get_generic_inner_type(r#type, "Option") {
            types.push(quote::quote!(std::option::Option<#inner_type>))
        } else if let Some(_) = get_field_macro_attr_path_value(field, "each") {
            if let Some(_) = get_generic_inner_type(r#type, "Vec") {
                types.push(quote::quote!(#r#type))
            } else {
                return Err(syn::Error::new(field.span(), "`each` field must be a Vec type"));
            }
        } else {
            types.push(quote::quote!(std::option::Option<#r#type>))
        }
    }

    Ok(quote::quote!(
        #(#idents: #types),*
    ))
}

pub(crate) fn generate_builder_method_fields(st: &syn::DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let fields = get_fields_from_derive_input(st)?;

    let mut builder_clauses = vec![];
    for field in fields.iter() {
        let ident = &field.ident;
        let r#type = &field.ty;
        if let Some(_) = get_field_macro_attr_path_value(field, "each") {
            if let Some(_) = get_generic_inner_type(r#type, "Vec") {
                builder_clauses.push(quote::quote!(
                    #ident: std::vec::Vec::new(),
                ))
            } else {
                return Err(syn::Error::new(field.span(), "`each` field must be a Vec type"));
            }
        } else {
            builder_clauses.push(quote::quote!(
                #ident: std::option::Option::None,
            ))
        }
    }

    Ok(quote::quote!(
        #(#builder_clauses)*
    ))
}

pub(crate) fn generate_builder_setter_methods(st: &syn::DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let fields = get_fields_from_derive_input(st)?;

    let mut build_setter_methods = vec![];
    for field in fields.iter() {
        let ident = &field.ident;
        let r#type = &field.ty;

        if let Some(inner_type) = get_generic_inner_type(r#type, "Option") {
            build_setter_methods.push(quote::quote!(
                fn #ident(&mut self, #ident: #inner_type) -> &mut Self {
                    self.#ident = std::option::Option::Some(#ident);
                    self
                }
            ))
        } else if let Some(ref user_ident) = get_field_macro_attr_path_value(field, "each") {
            if let Some(inner_type) = get_generic_inner_type(r#type, "Vec") {
                let mut token_stream = proc_macro2::TokenStream::new();
                token_stream.extend(quote::quote!(
                    fn #user_ident(&mut self, #user_ident: #inner_type) -> &mut Self {
                        self.#ident.push(#user_ident);
                        self
                    }
                ));
                if Some(user_ident) != ident.as_ref() {
                    token_stream.extend(quote::quote!(
                        fn #ident(&mut self, #ident: #r#type) -> &mut Self {
                            self.#ident = #ident;
                            self
                        }
                    ));
                }
                build_setter_methods.push(token_stream)
            } else {
                return Err(syn::Error::new(field.span(), "`each` field must be a Vec type"));
            }
        } else {
            build_setter_methods.push(quote::quote!(
                fn #ident(&mut self, #ident: #r#type) -> &mut Self {
                    self.#ident = std::option::Option::Some(#ident);
                    self
                }
            ))
        }
    }

    Ok(quote::quote!(
        #(#build_setter_methods)*
    ))
}

pub(crate) fn generate_builder_build_method(st: &syn::DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let fields = get_fields_from_derive_input(st)?;

    let build_validate_pieces: Vec<_> = fields
        .iter()
        .filter_map(|field| {
            let ident = &field.ident;
            let r#type = &field.ty;
            if get_generic_inner_type(r#type, "Option").is_some() || get_field_macro_attr_path_value(field, "each").is_some() {
                None
            } else {
                Some(quote::quote!(
                    if self.#ident.is_none() {
                        let err = format!("{} field is missing", stringify!(#ident));
                        return std::result::Result::Err(err.into());
                    }
                ))
            }
        })
        .collect();

    let build_assign_pieces: Vec<_> = fields
        .iter()
        .map(|field| {
            let ident = &field.ident;
            let r#type = &field.ty;

            if get_generic_inner_type(r#type, "Option").is_some() || get_field_macro_attr_path_value(field, "each").is_some() {
                quote::quote!(
                    #ident: self.#ident.clone(),
                )
            } else {
                quote::quote!(
                    #ident: self.#ident.clone().unwrap(),
                )
            }
        })
        .collect();

    let struct_ident = &st.ident;
    Ok(quote::quote!(
        pub fn build(&mut self) -> std::result::Result<#struct_ident, std::boxed::Box<dyn std::error::Error>> {
            #(#build_validate_pieces)*

            let ret = #struct_ident {
                #(#build_assign_pieces)*
            };
            std::result::Result::Ok(ret)
        }
    ))
}
