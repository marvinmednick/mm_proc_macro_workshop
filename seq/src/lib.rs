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
    /// Repeats the given code the number of times replacing the identifier provided
    /// with the iteration number
    fn expand(&self) -> proc_macro2::TokenStream {
        // get the start and end of the range
        let begin = self.start.base10_parse::<u16>().unwrap();
        let end = self.end.base10_parse::<u16>().unwrap();
        // parse each entry and replace the given identifier with the iteration number
        let expanded: Vec<_> = (begin..end)
            .map(|i| self.expand_ts(self.contents.clone(), self.name.clone(), i.into()))
            .collect();

        quote! { #(#expanded)*  }
    }

    /// Expands a TokenStream2 by iterating through each of the TokenTrees and
    /// processing them  (TokenStream is essentially a list of TokenTrees and can be iterated over)
    fn expand_ts(
        &self,
        ts: proc_macro2::TokenStream,
        name: Ident,
        replace_value: u16,
    ) -> proc_macro2::TokenStream {
        let mut output = quote!{};
        for tt in ts.into_iter() {
            output.extend(self.expand_tt(tt, name.clone(), replace_value.clone()));
        }

        output
    }

    /// Expands a Token Tree
    /// Identifes the type of Token Tree and handle as needed
    /// Groups recursively processed via expand_ts, Idents are compared to the
    /// one were looking for, everything is used as is.
    fn expand_tt(
        &self,
        tt: proc_macro2::TokenTree,
        name: Ident,
        replace_value: u16,
    ) -> proc_macro2::TokenStream {
        let updated_tt = match tt {
            // Groups contain a TokenStream2 which must be recursively parsed
            // As such we'll create a new group and and then parse the Groups TokenStream
            // keeping the origin delimeter and span
            proc_macro2::TokenTree::Group(ref g) => {
                let updated_stream = self.expand_ts(g.stream(),name,replace_value);
                let mut new_group = proc_macro2::Group::new(g.delimiter(),updated_stream);
                new_group.set_span(g.span());
                proc_macro2::TokenTree::Group(new_group)
            } ,
            // Idents need to check to see if they match the ident provided in the original Macro
            // call
            proc_macro2::TokenTree::Ident(ref id) => {
                // If the ident matches
                if *id == name {
                    // need to replace the ident with a number
                    // The test is specifically looking for a unsuffixed value (i.e. just 5 instead of
                    // 5u64, 5u16. etc
                    // create a new value as a proc_macro2::Literal
                    let mut replace_lit = proc_macro2::Literal::u16_unsuffixed(replace_value);
                    // keep the same span
                    replace_lit.set_span(id.span());
                    //and convert it to a proc_macro2::TokenTree item 
                    proc_macro2::TokenTree::Literal(replace_lit)
                }
                else {
                    // nothing to on these (Punct or Literal)
                    tt
                }
                
            },
            tt => {
                tt
            },
        };
        std::iter::once(updated_tt).collect()
    }
}

#[proc_macro]
pub fn seq(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let parsed = parse_macro_input!(input as Seq);
    parsed.expand().into()
}
