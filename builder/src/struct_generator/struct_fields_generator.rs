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

pub(crate) fn generate(st: &syn::DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let fields = get_fields_from_derive_input(st)?;

    let idents: Vec<_> = fields.iter().map(|f| &f.ident).collect();
    let types: Vec<_> = fields.iter().map(|field| &field.ty).collect();

    Ok(quote::quote!(
        #(#idents: std::option::Option<#types>),*
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
