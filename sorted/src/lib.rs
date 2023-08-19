use proc_macro::TokenStream;
use quote::ToTokens;

#[proc_macro_attribute]
pub fn sorted(args: TokenStream, input: TokenStream) -> TokenStream {
    let _ = args;
    let _ = input;

    let st = syn::parse_macro_input!(input as syn::Item);
    match do_expand(&st) {
        Ok(token_stream) => token_stream.into(),
        Err(e) => {
            let mut t = e.to_compile_error();
            t.extend(st.to_token_stream());
            t.into()
        }
    }
}

fn do_expand(st: &syn::Item) -> syn::Result<proc_macro2::TokenStream> {
    match st {
        syn::Item::Enum(e) => check_enum_order(e),
        _ => syn::Result::Err(syn::Error::new(proc_macro2::Span::call_site(), "expected enum or match expression")),
    }
}

fn check_enum_order(st: &syn::ItemEnum) -> syn::Result<proc_macro2::TokenStream> {
    let origin_order: Vec<_> = st.variants.iter().map(|f| (f.ident.to_string(), f)).collect();
    let mut sorted = origin_order.clone();
    sorted.sort_by(|a, b| a.0.cmp(&b.0));
    for (a, b) in origin_order.iter().zip(sorted) {
        if a.0 != b.0 {
            return syn::Result::Err(syn::Error::new_spanned(&b.1.ident, format!("{} should sort before {}", b.0, a.0)));
        }
    }

    Ok(st.into_token_stream())
}
