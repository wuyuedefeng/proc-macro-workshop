use proc_macro::TokenStream;
use quote::ToTokens;
use syn::{spanned::Spanned, visit_mut::VisitMut};

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

#[proc_macro_attribute]
pub fn check(_args: TokenStream, input: TokenStream) -> TokenStream {
    let mut st = syn::parse_macro_input!(input as syn::ItemFn);
    match do_match_expand(&mut st) {
        Ok(token_stream) => token_stream.into(),
        Err(e) => {
            let mut t = e.to_compile_error();
            t.extend(st.to_token_stream());
            t.into()
        }
    }
}

fn do_match_expand(st: &mut syn::ItemFn) -> syn::Result<proc_macro2::TokenStream> {
    let mut visitor = MatchVistor { err: None };
    visitor.visit_item_fn_mut(st);
    match visitor.err {
        Some(err) => syn::Result::Err(err),
        _ => syn::Result::Ok(st.to_token_stream()),
    }
}

struct MatchVistor {
    err: Option<syn::Error>,
}
impl syn::visit_mut::VisitMut for MatchVistor {
    fn visit_expr_match_mut(&mut self, i: &mut syn::ExprMatch) {
        let mut target_idx = None;
        for (idx, attr) in i.attrs.iter().enumerate() {
            if get_path_string(&attr.path()) == "sorted" {
                target_idx = Some(idx);
                break;
            }
        }

        match target_idx {
            Some(target_idx) => {
                i.attrs.remove(target_idx);

                let mut match_arm_names: Vec<(_, &dyn ToTokens)> = Vec::new();
                for arm in &i.arms {
                    match &arm.pat {
                        syn::Pat::Path(p) => match_arm_names.push((get_path_string(&p.path), &p.path)),
                        syn::Pat::TupleStruct(p) => match_arm_names.push((get_path_string(&p.path), &p.path)),
                        syn::Pat::Struct(p) => match_arm_names.push((get_path_string(&p.path), &p.path)),
                        syn::Pat::Ident(i) => {
                            match_arm_names.push((i.ident.to_string(), &i.ident));
                        }
                        syn::Pat::Wild(w) => match_arm_names.push(("_".to_string(), &w.underscore_token)),
                        _ => {
                            self.err = Some(syn::Error::new(arm.pat.span(), "unsupported by #[sorted]"));
                            return;
                        }
                    }
                }

                let mut sorted_names = match_arm_names.clone();
                sorted_names.sort_by(|a, b| a.0.cmp(&b.0));
                for (a, b) in match_arm_names.iter().zip(sorted_names) {
                    if a.0 != b.0 {
                        self.err = Some(syn::Error::new_spanned(b.1, format!("{} should sort before {}", b.0, a.0)));
                        return;
                    }
                }
            }
            None => (),
        }
        syn::visit_mut::visit_expr_match_mut(self, i);
    }
}

fn get_path_string(p: &syn::Path) -> String {
    let mut buf = Vec::new();
    for s in &p.segments {
        buf.push(s.ident.to_string());
    }
    buf.join("::")
}
