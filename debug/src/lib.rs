mod common;

use proc_macro::TokenStream;

#[proc_macro_derive(CustomDebug, attributes(debug))]
pub fn derive(input: TokenStream) -> TokenStream {
    let st = syn::parse_macro_input!(input as syn::DeriveInput);
    match do_expand(&st) {
        Ok(token_stream) => token_stream.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

fn do_expand(st: &syn::DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    Ok(generate_debug_trait(st)?)
}

fn generate_debug_trait(st: &syn::DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let struct_name_ident = &st.ident;
    let fields = common::get_fields_from_derive_input(st)?;

    let mut debug_body_stream = proc_macro2::TokenStream::new();
    debug_body_stream.extend(quote::quote!(fmt.debug_struct(stringify!(#struct_name_ident))));
    for field in fields.iter() {
        let ident = &field.ident;
        // let r#type = &field.ty;

        let mut format_string = String::from("{:?}");
        if let Some(format) = common::get_field_macro_attr_path_value_string(field, "debug", Some(vec!["debug"]))? {
            format_string = format.to_string();
        }

        debug_body_stream.extend(quote::quote!(
            .field(stringify!(#ident), &format_args!(#format_string, &self.#ident))
        ));
    }
    debug_body_stream.extend(quote::quote!(
        .finish()
    ));

    Ok(quote::quote!(
        impl std::fmt::Debug for #struct_name_ident {
            fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
                #debug_body_stream
            }
        }
    ))
}
