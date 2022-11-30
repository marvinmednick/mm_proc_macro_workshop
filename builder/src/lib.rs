use proc_macro::{TokenStream};
use quote::quote;
use syn::{parse_macro_input,DeriveInput};

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let _input2 = input.clone();
    // let input1 = parse_macro_input!(input as DeriveInput);
    let test_type = quote!(String);

    let output2  = quote!( 
        struct Command2 { 
            test : usize,
            test2 : #test_type,
        }
        ).into();
    eprintln!("output is {:#?}",output2);
    let parsed_output = parse_macro_input!(output2 as DeriveInput);
    eprintln!("Parsed output is {:#?}",parsed_output);


    // Used in the quasi-quotation below as `#name`.

//    eprintln!("Input is {:#?}",input2);
//    eprintln!("........");
    //eprintln!("Parsed Input is {:#?}",input1);
    let output : proc_macro::TokenStream = quote!( 
        impl Command { 
            pub fn builder() { }
            pub fn test1() { }
        } 
        struct newOne {
            field1 : usize,
            fieedl2 : usize,
        }
        ).into();
    return output
}
