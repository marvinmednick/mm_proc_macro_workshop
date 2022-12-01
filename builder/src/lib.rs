use proc_macro::{TokenStream};
use quote::{quote,format_ident};
use syn::{parse_macro_input,DeriveInput};

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let _input2 = input.clone();
    let input3 = input.clone();

    let parsed_input : DeriveInput = syn::parse(input3).unwrap();
    let _parsed_input1 = parse_macro_input!(input as DeriveInput);
    
    let struct_name = parsed_input.ident;
    let builder_name = format_ident!("{}Builder",struct_name);
   // assert_eq!(parsed_input.ident,"Command");
    //let test_type = quote!(String);


//    eprintln!("Input is {:#?}",input2);
//    eprintln!("........");
    if let syn::Data::Struct(d) = parsed_input.data {
        //eprintln!("Parsed Input is {:#?}",d);
        if let syn::Fields::Named(_f) = d.fields {
            //eprintln!("named is {:#?}",f.named);
            //eprintln!("There are {} entries",f.named.len());
        //    for x in f.named {
         //       eprintln!("Field {}  type {:#?}",x.ident.unwrap(),x.ty);
          //  }
        }
        else {
            eprintln!("fields are {:#?}",d.fields);
        }
    }
    else {
        eprintln!("Did not match Parsed Input is {:#?}",parsed_input.data);
    }
    // eprintln!("Original input {:#?}",input2);
    let output : proc_macro::TokenStream = quote!( 
         pub struct #builder_name {
             executable: Option<String>,
             args: Option<Vec<String>>,
             env: Option<Vec<String>>,
             current_dir: Option<String>,
         }
         
        impl #struct_name { 
            pub fn builder() -> #builder_name {
                #builder_name {
                    executable: None,
                    args: None,
                    env: None,
                    current_dir: None,
                }
            }
        } 
        ).into();
    return output
}
