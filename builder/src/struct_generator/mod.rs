mod struct_fields_generator;

use syn::spanned::Spanned;
pub(crate) fn generate(st: &syn::DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let struct_name_ient = &st.ident;
    let struct_name_literal = struct_name_ient.to_owned();
    let struct_builder_name_literal = format!("{}Builder", struct_name_literal);
    let struct_builder_name_ident = syn::Ident::new(&struct_builder_name_literal, st.span());

    let struct_fields_ref = struct_fields_generator::generate(st)?;
    let struct_builder_method_fileds_ref = struct_fields_generator::generate_builder_method_fields(st)?;
    Ok(quote::quote!(
        pub struct #struct_builder_name_ident {
            #struct_fields_ref
        }

        impl #struct_name_ient {
            pub fn builder() -> #struct_builder_name_ident {
                #struct_builder_name_ident {
                    #struct_builder_method_fileds_ref
                }
            }
        }
    ))
}
