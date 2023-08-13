use proc_macro::TokenStream;

#[proc_macro]
pub fn seq(input: TokenStream) -> TokenStream {
    let st = syn::parse_macro_input!(input as SeqParser);

    let mut ret = proc_macro2::TokenStream::new();
    for i in st.start..st.end {
        ret.extend(st.expand(&st.body, i));
    }
    ret.into()
}

struct SeqParser {
    variable_ident: syn::Ident,
    start: isize,
    end: isize,
    body: proc_macro2::TokenStream,
}

impl syn::parse::Parse for SeqParser {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let variable_ident: syn::Ident = input.parse()?;
        input.parse::<syn::Token!(in)>()?;
        let start: syn::LitInt = input.parse()?;
        input.parse::<syn::Token!(..)>()?;
        let end: syn::LitInt = input.parse()?;
        let body_buff;
        syn::braced!(body_buff in input);
        let body: proc_macro2::TokenStream = body_buff.parse()?;
        Ok(SeqParser {
            variable_ident,
            start: start.base10_parse()?,
            end: end.base10_parse()?,
            body,
        })
    }
}

impl SeqParser {
    fn expand(&self, ts: &proc_macro2::TokenStream, n: isize) -> proc_macro2::TokenStream {
        let token_tree_vec = ts.clone().into_iter().collect::<Vec<_>>();
        let mut ret = proc_macro2::TokenStream::new();

        for idx in 0..token_tree_vec.len() {
            let token_tree = &token_tree_vec[idx];
            match token_tree {
                proc_macro2::TokenTree::Group(group) => {
                    let new_stream = self.expand(&group.stream(), n);
                    let wrap_in_group = proc_macro2::Group::new(group.delimiter(), new_stream);
                    ret.extend(quote::quote!(#wrap_in_group));
                }
                proc_macro2::TokenTree::Ident(ident) => {
                    if ident == &self.variable_ident {
                        let new_ident = proc_macro2::Literal::i64_unsuffixed(n as i64);
                        ret.extend(quote::quote!(#new_ident));
                    } else {
                        ret.extend(quote::quote!(#token_tree));
                    }
                }
                _ => ret.extend(quote::quote!(#token_tree)),
            }
        }

        ret
    }
}
