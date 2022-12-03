use proc_macro::TokenStream;
use quote::{quote,format_ident };
use syn::{parse_macro_input,DeriveInput};


#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let _input2 = input.clone();
    let input3 = input.clone();

    let parsed_input : DeriveInput = syn::parse(input3).unwrap();
    let _parsed_input1 = parse_macro_input!(input as DeriveInput);
    
    let struct_name = parsed_input.ident;
    let builder_name = format_ident!("{}Builder",struct_name);


    let mut my_field_name = Vec::<syn::Ident>::new();
    let mut my_field_type = Vec::<syn::Type>::new();

    if let syn::Data::Struct(d) = parsed_input.data {
        //eprintln!("Parsed Input is {:#?}",d);
        if let syn::Fields::Named(f) = d.fields {
            //eprintln!("named is {:#?}",f.named);
            //eprintln!("There are {} entries",f.named.len());
            //let mut i = 1;
            for x in f.named {
          //     eprintln!("Field number {}",i);
            //   i += 1;
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
//    let test1 : proc_macro::TokenStream = quote!(xyz : String).into();
    // eprintln!("Original input {:#?}",input2);
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

            fn build(&mut self) -> Result<#struct_name,  &'static str> {

                let mut missing_count = 0;
                #(if self.#my_field_name == None { missing_count +=1 };)*


                 let x = #struct_name {
                #(#my_field_name:  self.#my_field_name.clone().unwrap(),)*
//                    executable: self.executable.clone().unwrap(),
 //                   args: vec!["a".to_string(), "1".to_string()],
  //                  env: vec!["".to_string()],
   //                 current_dir: "".to_string(),
                 };

                if missing_count == 0 {
                    Ok(x)
                } 
                else {
//                    Ok(x)
 //                   Err(Box::new(::std::error::Error("Ooops".into())))
                   Err("Fields missing")
                }
            }

        }



        ).into();
    return output
}
