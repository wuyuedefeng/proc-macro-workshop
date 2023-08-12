mod struct_fields_generator;

use syn::spanned::Spanned;
pub(crate) fn generate(st: &syn::DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let struct_name_ident = &st.ident;
    let struct_name_literal = struct_name_ident.to_string();
    let struct_builder_name_literal = format!("{}Builder", struct_name_literal);
    let struct_builder_name_ident = syn::Ident::new(&struct_builder_name_literal, st.span());

    let struct_fields_ref = struct_fields_generator::generate(st)?;
    let struct_builder_method_fileds_ref = struct_fields_generator::generate_builder_method_fields(st)?;
    let struct_builder_setter_methods = struct_fields_generator::generate_builder_setter_methods(st)?;
    let struct_builder_build_method = struct_fields_generator::generate_builder_build_method(st)?;
    Ok(quote::quote!(
        pub struct #struct_builder_name_ident {
            #struct_fields_ref
        }

        impl #struct_builder_name_ident {
            #struct_builder_setter_methods

            #struct_builder_build_method
        }

        impl #struct_name_ident {
            pub fn builder() -> #struct_builder_name_ident {
                #struct_builder_name_ident {
                    #struct_builder_method_fileds_ref
                }
            }
        }
    ))
}
