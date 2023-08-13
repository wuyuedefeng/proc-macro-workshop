use proc_macro::TokenStream;

#[proc_macro]
pub fn seq(input: TokenStream) -> TokenStream {
    let st = syn::parse_macro_input!(input as SeqParser);

    return TokenStream::new();
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
