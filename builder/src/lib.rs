use proc_macro::{TokenStream, Ident};
use quote::{quote,format_ident, ToTokens};
use syn::{parse_macro_input,DeriveInput};

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let _input2 = input.clone();
    let input3 = input.clone();

    let parsed_input : DeriveInput = syn::parse(input3).unwrap();
    let _parsed_input1 = parse_macro_input!(input as DeriveInput);
    
    let struct_name = parsed_input.ident;
    let builder_name = format_ident!("{}Builder",struct_name);
    let test_name = format_ident!("{}Test",struct_name);
   // assert_eq!(parsed_input.ident,"Command");
    //let test_type = quote!(String);
    //


//    eprintln!("Input is {:#?}",input2);
//    eprintln!("........");
    let mut my_field_name = Vec::<syn::Ident>::new();
    let mut my_field_type = Vec::<syn::Type>::new();
    if let syn::Data::Struct(d) = parsed_input.data {
        //eprintln!("Parsed Input is {:#?}",d);
        if let syn::Fields::Named(f) = d.fields {
            //eprintln!("named is {:#?}",f.named);
            //eprintln!("There are {} entries",f.named.len());
            let mut i = 1;
            for x in f.named {
               eprintln!("Field number {}",i);
               i += 1;
//               eprintln!("Field {}  type {:#?}",x.ident.unwrap().as_ref(),x.ty.into_token_stream());
               my_field_name.push(x.ident.unwrap());
               my_field_type.push(x.ty);
            }
        }
        else {
            eprintln!("fields are {:#?}",d.fields);
        }
    }
    else {
        eprintln!("Did not match Parsed Input is {:#?}",parsed_input.data);
    }
    let test1 : proc_macro::TokenStream = quote!(xyz : String).into();
    eprintln!("test1 is {}",test1);
    // eprintln!("Original input {:#?}",input2);
    let output : proc_macro::TokenStream = quote!( 
         pub struct #builder_name {
            #(#my_field_name : Option<#my_field_type>),* ,
         }
         
        pub struct #test_name  {
            test_before: usize,
            #(#my_field_name : Option<#my_field_type>),* ,
            test_after: usize,
        }

        impl #struct_name { 
            pub fn builder() -> #builder_name {
                #builder_name {
                    #(#my_field_name : None ),* ,
                }
            }
        } 
        impl #builder_name {
            fn executable(&mut self, executable: String) -> &mut Self {
                self.executable = Some(executable);
                self
            }
        }


        ).into();
    return output
}
