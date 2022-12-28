#![feature(proc_macro_diagnostic)]
//use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream, Result};
use syn::{parse_macro_input, Ident, LitInt, Token};

struct Seq {
    name: Ident,
    start: LitInt,
    end: LitInt,
    contents: proc_macro2::TokenStream,
}

fn expand_ts(ts: proc_macro2::TokenStream, name: Ident, index: u64) -> proc_macro2::TokenStream {
    quote! { compile_error!(concat!("error number ", stringify!(#index))); }
}

impl Parse for Seq {
    fn parse(input: ParseStream) -> Result<Self> {
        //Expec  Ident, Token![in], LitInt, Token![..], LitInt.
        let name: Ident = input.parse()?;
        input.parse::<Token![in]>()?;
        let start: LitInt = input.parse()?;
        input.parse::<Token![..]>()?;
        let end: LitInt = input.parse()?;
        // now need to collect all the content
        let content;
        let _braces = syn::braced!(content in input);
        let contents = proc_macro2::TokenStream::parse(&content)?;
        Ok(Seq {
            name,
            start,
            end,
            contents,
        })
    }
}

impl Seq {
    fn expand(&self) -> proc_macro2::TokenStream {
        let a = self.start.base10_parse::<u16>().unwrap();
        let b = self.end.base10_parse::<u16>().unwrap();
        let expanded: Vec<_> = (a..b)
            .map(|i| self.expand_ts(self.contents.clone(), self.name.clone(), i.into()))
            .collect();

        quote! { #(#expanded)*  }
    }

    fn expand_ts(
        &self,
        ts: proc_macro2::TokenStream,
        name: Ident,
        index: u64,
    ) -> proc_macro2::TokenStream {
        for tt in ts.into_iter() {
            self.expand_tt(tt, name.clone(), index.clone());
        }

        quote! { compile_error!(concat!("error number ", stringify!(#index))); }
    }

    fn expand_tt(
        &self,
        tt: proc_macro2::TokenTree,
        name: Ident,
        replace_value: u64,
    ) -> proc_macro2::TokenStream {
        let updated_tt = match tt {
            proc_macro2::TokenTree::Group(ref g) => tt,
            proc_macro2::TokenTree::Ident(ref id) => {
                let mut replace_lit = proc_macro2::Literal::u64_unsuffixed(replace_value);
                replace_lit.set_span(id.span());
                proc_macro2::TokenTree::Literal(replace_lit)
            }
            tt => tt,
        };
        std::iter::once(updated_tt).collect()
    }
}

#[proc_macro]
pub fn seq(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let parsed = parse_macro_input!(input as Seq);
    parsed.expand().into()
}
