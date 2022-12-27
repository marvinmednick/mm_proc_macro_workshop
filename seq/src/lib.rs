#![feature(proc_macro_diagnostic)]
use proc_macro::TokenStream;
use syn::parse::{Parse, ParseStream, ParseBuffer,Result};
use quote::{quote, format_ident} ;
use syn::{parse_macro_input, Ident, LitInt, Token, };

struct Seq { 
    name : Ident,
    start: LitInt,
    end: LitInt,
    contents: proc_macro2::TokenStream,
}

impl Parse for Seq {

    fn parse(input: ParseStream) -> Result<Self> {
            
 //Expec  syn::Ident, Token![in], syn::LitInt, Token![..], syn::LitInt.
        let name : Ident  = input.parse()?;
        input.parse::<Token![in]>()?;
        let start : LitInt  = input.parse()?;
        input.parse::<Token![..]>()?;
        let end : LitInt  = input.parse()?;
        // now need to collect all the content
        let content;
        let _braces = syn::braced!(content in input);
        let contents = proc_macro2::TokenStream::parse(&content)?;
//        eprintln!("Content is {:?}",content);
//        eprintln!("Parsed content is {:?}",contents);
        Ok(Seq { name, start, end, contents})

    }
}

#[proc_macro]
pub fn seq(input: TokenStream) -> TokenStream {
 //   let _ = input;

//    eprintln!("Input is {:#?}", input);

    //let parsed = parse_macro_input!(input );
    
    let parsed = parse_macro_input!(input as Seq);
    eprintln!("Name is {} Start is {} End is {} TT is {:#?} ",parsed.name, parsed.start, parsed.end, parsed.contents);
//    let content = quote!{ #contents };

    let mut new = quote! {};

    let a = parsed.start.base10_parse::<u16>().unwrap();
    let b = parsed.end.base10_parse::<u16>().unwrap();
    for n in a..b {
        let sname = format_ident!("xyz_{}",n);
        let content = quote!{ struct xyz { i : u32 } };
        new.extend(content);
    }
        

    quote! { #new  }.into()
}
