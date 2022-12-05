use proc_macro::TokenStream;
use quote::{quote,format_ident };
use syn::DeriveInput;


#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let input3 = input.clone();

    let parsed_input : DeriveInput = syn::parse(input3).unwrap();
    
    let struct_name = parsed_input.ident;
    let builder_name = format_ident!("{}Builder",struct_name);


    let mut my_field_name = Vec::<syn::Ident>::new();
    let mut my_field_name_strings = Vec::<String>::new();
    let mut my_field_type = Vec::<syn::Type>::new();

    if let syn::Data::Struct(d) = parsed_input.data {
        if let syn::Fields::Named(f) = d.fields {
            for x in f.named {
               my_field_name.push(x.clone().ident.unwrap());
               my_field_name_strings.push(format!("{}",x.ident.unwrap()));
               my_field_type.push(x.ty);
            }
        }
        else {
            eprintln!("fields are {:#?}",d.fields);
        }
    }
    else {
        eprintln!("Did not match Parsed Input is {:#?}",parsed_input.data);
//
    }
    let output : proc_macro::TokenStream = quote!( 
         pub struct #builder_name {
            #(#my_field_name : Option<#my_field_type>),* ,
         }
         
        impl #struct_name { 
            pub fn builder() -> #builder_name {
                #builder_name {
                    #(#my_field_name : None ),* ,
                }
            }
        } 

        impl #builder_name {
            #(fn #my_field_name (&mut self, #my_field_name: #my_field_type) -> &mut Self {
                self.#my_field_name = Some(#my_field_name);
                self
            }

            )* 

            fn build(&mut self) -> Result<#struct_name,  Box<dyn std::error::Error>> {

                let mut missing_fields = Vec::<String>::new();
                #(
                    if self.#my_field_name == None {
                        missing_fields.push(#my_field_name_strings.to_string());
//                        missing_fields.push(std::stringify!(#my_field_name).to_string);
                    };
                )*



                if missing_fields.len() == 0 {
                    let x = #struct_name {
                       #(#my_field_name:  self.#my_field_name.clone().unwrap(),)*
                    };

                    Ok(x)
                } 
                else {
                    let missing_list = missing_fields.join(",");
                    let err = format!("The following fields are not yet set: {}",missing_list);
                    return std::result::Result::Err(err.into())

                }
            }

        }



        ).into();
    return output
}
