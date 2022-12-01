use proc_macro::{TokenStream};
use quote::quote;
use syn::{parse,parse_macro_input,DeriveInput};

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let input2 = input.clone();
    let input3 = input.clone();

    let parsed_input : DeriveInput = syn::parse(input3).unwrap();
    let parsed_input1 = parse_macro_input!(input as DeriveInput);
    
    

    assert_eq!(parsed_input.ident,"Command");
    let test_type = quote!(String);


//    eprintln!("Input is {:#?}",input2);
//    eprintln!("........");
    if let syn::Data::Struct(d) = parsed_input.data {
        //eprintln!("Parsed Input is {:#?}",d);
        if let syn::Fields::Named(f) = d.fields {
            //eprintln!("named is {:#?}",f.named);
            eprintln!("There are {} entries",f.named.len());
            for x in f.named {
                eprintln!("Field {}  type {:#?}",x.ident.unwrap(),x.ty);
            }
        }
        else {
            eprintln!("fields are {:#?}",d.fields);
        }
    }
    else {
        eprintln!("Did not match Parsed Input is {:#?}",parsed_input.data);
    }
    eprintln!("Original input {:#?}",input2);
    let output : proc_macro::TokenStream = quote!( 
        impl Command { 
            pub fn builder() { }
            pub fn test1() { }
        } 
        struct newOne {
            field1 : usize,
            field2 : #test_type,
        }
        ).into();
    return output
}
