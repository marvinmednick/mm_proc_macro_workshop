use proc_macro::TokenStream;
use quote::{quote,format_ident };
use syn::DeriveInput;

fn get_type_info(ty : syn::Type) -> (syn::Type, bool) {
    let mut is_option = false;
    if let syn::Type::Path(type_path) = ty.clone() {
        let num_segs = type_path.path.segments.len();
        /*
        eprintln!("Num of segs {:?}",num_segs);
        for i in 0..num_segs {
            eprintln!("seg is {:?}",type_path.path.segments[i]);
            let www  = type_path.path.segments[i].clone() ;
            eprintln!("www ident   is {:?}",www.ident);
            if www.ident == "Option" {
                eprintln!("Found option!!!");
            }
            eprintln!("www arguments  is {:?}",www.arguments);
        }
        eprintln!("last seg is {:#?}",type_path.path.segments.last());
        */
        if let Some(seg) = type_path.path.segments.last() {
            if seg.ident == "Option" {
                eprintln!("Found option!!!");
                if let syn::PathArguments::AngleBracketed(
                    syn::AngleBracketedGenericArguments {
                        ref args,
                        ..
                    }
                ) = seg.arguments {
                    if let Some(syn::GenericArgument::Type(inner_type)) = args.first() {
                        eprintln!("with Inner type is {:#?}", inner_type);
                    }
                }

            }
        }

    }
    (ty, is_option)
}

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let input3 = input.clone();

    let parsed_input : DeriveInput = syn::parse(input3).unwrap();
    
    
    let struct_name = parsed_input.ident;
    let builder_name = format_ident!("{}Builder",struct_name);


    let mut my_field_name = Vec::<syn::Ident>::new();
    let mut my_field_type = Vec::<syn::Type>::new();

    if let syn::Data::Struct(d) = parsed_input.data {
        if let syn::Fields::Named(f) = d.fields {
            eprintln!("fields are {:#?}",f.named);
            for x in f.named {
               my_field_name.push(x.clone().ident.unwrap());
               let (updated_type, is_option) = get_type_info(x.ty);
               my_field_type.push(updated_type);
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
                        missing_fields.push(std::stringify!(#my_field_name).to_string());
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
