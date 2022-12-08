use proc_macro::TokenStream;
use quote::{quote,format_ident};
use syn::DeriveInput;

fn unwrapped_option_type<'a>(ty : &'a syn::Type) -> Option<&'a syn::Type> {

    // check that path is of a type 
    if let syn::Type::Path(type_path) = ty {

        // default return to None
        // get the last segment
        if let Some(seg) = type_path.path.segments.last() {
            // check if its not
            if seg.ident == "Option" {
//                eprintln!("Found option!!!");
                if let syn::PathArguments::AngleBracketed(
                    syn::AngleBracketedGenericArguments {
                        ref args,
                        ..
                    }
                ) = seg.arguments {
                    if let Some(syn::GenericArgument::Type(inner_type)) = args.first() {
 //                       eprintln!("with Inner type is {:#?}", inner_type);
                        return Some(inner_type)
                    }
                }

            }
        }

    }
    // default to None if doesn't match
    return None
}

#[proc_macro_derive(Builder, attributes(builder))]
pub fn derive(input: TokenStream) -> TokenStream {
    let input3 = input.clone();

    let parsed_input : DeriveInput = syn::parse(input3).unwrap();
    eprintln!("parsed input data {:?}",parsed_input.data);
    
    
    let struct_name = parsed_input.ident;
    let builder_name = format_ident!("{}Builder",struct_name);


    let mut my_field_name = Vec::<syn::Ident>::new();
    let mut my_field_type = Vec::<syn::Type>::new();
    let mut my_field_value = Vec::<proc_macro2::TokenStream>::new();



    let fields = if let syn::Data::Struct(
        syn::DataStruct {
            fields: syn::Fields::Named(syn::FieldsNamed {
                ref named, ..
                }),
            ..
        }
    ) = parsed_input.data { named }
    else {
        unimplemented!();
    };

    if let syn::Data::Struct(d) = parsed_input.data {
        if let syn::Fields::Named(f) = d.fields {
//            eprintln!("fields are {:#?}",f.named);
            for x in f.named {
               let cur_name = x.clone().ident.unwrap();
               my_field_name.push(cur_name.clone());
               eprintln!("Field {:?}",cur_name);
               eprintln!("Field attributes {:#?}",x.attrs);

               let updated = unwrapped_option_type(&x.ty);
               if let Some(updated_type) = updated {
                   my_field_value.push(quote!(self.#cur_name.clone()));
                   my_field_type.push(updated_type.clone());
                }
                else {
                   my_field_value.push(quote!(self.#cur_name.clone().unwrap()));
                   my_field_type.push(x.ty);
                }
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

                if missing_fields.len() == 0 || true {
//                    let z = #my_field_value ;
                    let x = #struct_name {
                       //#(#my_field_name:  if #my_field_optional { self.#my_field_name.clone() } else { self.#my_field_name.clone().unwrap() } ,)*
//                       #(#my_field_name:    self.#my_field_name.clone().unwrap() ,)*
                       #(#my_field_name:    #my_field_value ,)*
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
