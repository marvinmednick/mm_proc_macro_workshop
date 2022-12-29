#![feature(proc_macro_diagnostic)]
use std::ops::Range;

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
        let range = begin..end;
        let expanded_sequence =
            expand_sequences(self.contents.clone(), self.name.clone(), range.clone());

        //eprintln!("TS is {:#?}", self.contents);
        // parse each entry and replace the given identifier with the iteration number
        /*
        let expanded: Vec<_> = range
            .map(|i| expand_ts(self.contents.clone(), self.name.clone(), i.into()))
            .collect();
        quote! { #(#expanded)*  }
        */

        quote! { #expanded_sequence  }
    }
}

/// Parse the Transport stream looking for
fn expand_sequences(
    ts: proc_macro2::TokenStream,
    name: Ident,
    range: Range<u16>,
) -> proc_macro2::TokenStream {
    let mut output = quote! {};
    let mut ts_iter = ts.into_iter();
    while let Some(tt) = ts_iter.next() {
        eprintln!("expand seq processing {:?}", tt);
        output.extend(expand_repeat_group(
            tt,
            &mut ts_iter,
            name.clone(),
            range.clone(),
        ));
    }
    output
}

fn expand_repeat_group(
    tt: proc_macro2::TokenTree,
    ts_iter: &mut proc_macro2::token_stream::IntoIter,
    name: Ident,
    range: Range<u16>,
) -> proc_macro2::TokenStream {
    let mut peek_iter = ts_iter.clone();
    let next1 = peek_iter.next();
    let next2 = peek_iter.next();
    eprintln!(
        "Repeat group checking on -----\n {:?}, {:?}, {:?}\n---End repeat group",
        tt, next1, next2
    );
    let output = match (tt.clone(), next1, next2) {
        (
            proc_macro2::TokenTree::Punct(punct),
            Some(proc_macro2::TokenTree::Group(g)),
            Some(proc_macro2::TokenTree::Punct(punct1)),
        ) if punct.as_char() == '#'
            && g.delimiter() == proc_macro2::Delimiter::Parenthesis
            && punct1.as_char() == '*' =>
        {
            eprintln!("Found Group {:?}", g.stream());
            let expanded: Vec<_> = range
                .map(|i| expand_ts(g.stream(), name.clone(), i.into()))
                .collect();

            *ts_iter = peek_iter;
            quote! { #(#expanded)*  }
        }
        // other wise if we find a group we need to recursively parse it, continuing to look
        // for a seuqnce to repeat
        (proc_macro2::TokenTree::Group(ref g), _, _) => {
            let updated_stream = expand_sequences(g.stream(), name, range);
            let mut new_group = proc_macro2::Group::new(g.delimiter(), updated_stream);
            new_group.set_span(g.span());
            std::iter::once(proc_macro2::TokenTree::Group(new_group)).collect()
        }
        _ => std::iter::once(tt).collect(),
    };
    output
}

/// Expands a TokenStream2 by iterating through each of the TokenTrees and
/// processing them  (TokenStream is essentially a list of TokenTrees and can be iterated over)
fn expand_ts(
    ts: proc_macro2::TokenStream,
    name: Ident,
    replace_value: u16,
) -> proc_macro2::TokenStream {
    let mut output = quote! {};
    let mut ts_iter = ts.into_iter();
    while let Some(tt) = ts_iter.next() {
        output.extend(expand_tt(
            tt,
            &mut ts_iter,
            name.clone(),
            replace_value.clone(),
        ));
    }

    output
}

/// Expands a Token Tree
/// Identifes the type of Token Tree and handle as needed
/// Groups recursively processed via expand_ts, Idents are compared to the
/// one were looking for, everything is used as is.
fn expand_tt(
    tt: proc_macro2::TokenTree,
    ts_iter: &mut proc_macro2::token_stream::IntoIter,
    name: Ident,
    replace_value: u16,
) -> proc_macro2::TokenStream {
    let updated_tt = match tt {
        // Groups contain a TokenStream2 which must be recursively parsed
        // As such we'll create a new group and and then parse the Groups TokenStream
        // keeping the origin delimeter and span
        proc_macro2::TokenTree::Group(ref g) => {
            let updated_stream = expand_ts(g.stream(), name, replace_value);
            let mut new_group = proc_macro2::Group::new(g.delimiter(), updated_stream);
            new_group.set_span(g.span());
            proc_macro2::TokenTree::Group(new_group)
        }
        // Idents need to check to see if they match the ident provided in the original Macro
        // call if the ident matches
        proc_macro2::TokenTree::Ident(id) if id == name => {
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
        // Need to look ahead for the pattern Ident ~ N  e.g. f~N to convet to f0, f1 etc
        // (and optionally ident~N~ and ident~N~ident  which would convert f~N~ to f0 and f~N~b  to
        // f0b  (f~N~ could also be considered an error)
        proc_macro2::TokenTree::Ident(ref id) => {
            let mut peek_iter = ts_iter.clone(); // get a copy of the stream to peeks and consume
                                                 // until we figure out what we want to do

            match (peek_iter.next(), peek_iter.next()) {
                (
                    Some(proc_macro2::TokenTree::Punct(punct)),
                    Some(proc_macro2::TokenTree::Ident(id2)),
                ) if punct.as_char() == '~' && id2 == name => {
                    // At this point we know we have id~N at least, though
                    // there could still be ida~N~idb
                    let mut peek_cpy = peek_iter.clone();
                    match (peek_cpy.next(), peek_cpy.next()) {
                        (
                            Some(proc_macro2::TokenTree::Punct(punct)),
                            Some(proc_macro2::TokenTree::Ident(id3)),
                        ) if punct.as_char() == '~' => {
                            // at this pint we have idA~N~idB e.g f~_bar -> f1_bar
                            let new_ident_name = format!("{}{}{}", id, replace_value, id3);
                            //eprintln!("New identifier is {}", new_ident_name);
                            *ts_iter = peek_cpy;
                            proc_macro2::TokenTree::Ident(proc_macro2::Ident::new(
                                &new_ident_name,
                                id.span(),
                            ))
                        }

                        (_, _) => {
                            // otherwise we just ahve id~N
                            let new_ident_name = format!("{}{}", id, replace_value);
                            //eprintln!("New identifier is {}", new_ident_name);
                            *ts_iter = peek_iter;
                            proc_macro2::TokenTree::Ident(proc_macro2::Ident::new(
                                &new_ident_name,
                                id.span(),
                            ))
                        }
                    }
                }

                (_, _) => tt,
            }
        }
        _ => tt,
    };
    std::iter::once(updated_tt).collect()
}

#[proc_macro]
pub fn seq(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let parsed = parse_macro_input!(input as Seq);
    parsed.expand().into()
}
