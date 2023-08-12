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

fn get_optional_inner_type(r#type: &syn::Type) -> Option<&syn::Type> {
    if let syn::Type::Path(syn::TypePath { path: syn::Path { segments, .. }, .. }) = r#type {
        if let Some(seg) = segments.last() {
            if seg.ident.to_string() == "Option" {
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

pub(crate) fn generate(st: &syn::DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let fields = get_fields_from_derive_input(st)?;

    let idents: Vec<_> = fields.iter().map(|field| &field.ident).collect();
    let types: Vec<_> = fields
        .iter()
        .map(|field| {
            let r#type = &field.ty;
            if let Some(inner_type) = get_optional_inner_type(r#type) {
                quote::quote!(std::option::Option<#inner_type>)
            } else {
                quote::quote!(std::option::Option<#r#type>)
            }
        })
        .collect();

    Ok(quote::quote!(
        #(#idents: #types),*
    ))
}

pub(crate) fn generate_builder_method_fields(st: &syn::DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let fields = get_fields_from_derive_input(st)?;

    let builder_clauses: Vec<_> = fields
        .iter()
        .map(|field| {
            let ident = &field.ident;
            quote::quote!(
                #ident: std::option::Option::None
            )
        })
        .collect();

    Ok(quote::quote!(
        #(#builder_clauses),*
    ))
}

pub(crate) fn generate_builder_setter_methods(st: &syn::DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let fields = get_fields_from_derive_input(st)?;

    let build_setter_methods: Vec<_> = fields
        .iter()
        .map(|field| {
            let ident = &field.ident;
            let r#type = &field.ty;
            if let Some(inner_type) = get_optional_inner_type(r#type) {
                quote::quote!(
                    fn #ident(&mut self, #ident: #inner_type) -> &mut Self {
                        self.#ident = std::option::Option::Some(#ident);
                        self
                    }
                )
            } else {
                quote::quote!(
                    fn #ident(&mut self, #ident: #r#type) -> &mut Self {
                        self.#ident = std::option::Option::Some(#ident);
                        self
                    }
                )
            }
        })
        .collect();

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
            if let Some(_) = get_optional_inner_type(r#type) {
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

            if let Some(_) = get_optional_inner_type(r#type) {
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
