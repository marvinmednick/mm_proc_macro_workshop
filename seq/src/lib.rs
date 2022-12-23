#![feature(proc_macro_diagnostic)]
use proc_macro::TokenStream;
use syn::parse::{Parse, ParseStream, ParseBuffer,Result};
use quote::quote ;
use syn::{parse_macro_input, Ident, LitInt, Token, };

struct Seq { 
    name : Ident,
    start: LitInt,
    end: LitInt,
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
        let _tt = proc_macro2::TokenStream::parse(&content)?;
        eprintln!("Content is {:?}",content);
        eprintln!("Parsed content is {:?}",_tt);
        Ok(Seq { name, start, end, })

    }
}

#[proc_macro]
pub fn seq(input: TokenStream) -> TokenStream {
 //   let _ = input;

//    eprintln!("Input is {:#?}", input);

    //let parsed = parse_macro_input!(input );
    
    let Seq {
        name,
        start, 
        end,
    } = parse_macro_input!(input as Seq);
    eprintln!("Name is {} Start is {} End is {}",name, start, end);


    quote! { }.into()
}
