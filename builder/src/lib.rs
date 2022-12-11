use proc_macro::TokenStream;
use quote::{quote,format_ident};
use syn::{DeriveInput, PathSegment};

fn unwrapped_option_type<'a>(ty : &'a syn::Type) -> Option<&'a syn::Type> {

    // check that path is of a type 
    
    if let syn::Type::Path(type_path) = ty {

        // default return to None
        // get the last segment
        if let Some(seg) = type_path.path.segments.last() {
            // check if its not
            if seg.ident == "Option" {
                if let syn::PathArguments::AngleBracketed(
                    syn::AngleBracketedGenericArguments {
                        ref args,
                        ..
                    }
                ) = seg.arguments {
                    if let Some(syn::GenericArgument::Type(inner_type)) = args.first() {
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
    let parsed_copy = parsed_input.clone();

//    eprintln!("Parsed Tree {:#?}",parsed_copy);
//    eprintln!("Parsed Tree -------- END");

    
    let struct_name = parsed_input.ident;
    let builder_name = format_ident!("{}Builder",struct_name);


    let fields = if let syn::Data::Struct(
        syn::DataStruct {
            fields: syn::Fields::Named(syn::FieldsNamed {
                ref named, ..
                }),
            ..
        }
    ) = parsed_copy.data { named }
    else {
        unimplemented!();
    };

    // builder structure fields
    let builder_def_fields = fields.iter().map(|f| {
       let name = &f.ident;
       let attr_list = &f.attrs;
//       eprintln!("NEW Field {:?}  len Attr: {} ATTR: {:#?}",name, attr_list.len(),attr_list);
       let ty = match  unwrapped_option_type(&f.ty) {
           Some(updated) => updated,
           None => &f.ty,
        };


       for a in &f.attrs {
           let path = &a.path;
           let tokens = &a.tokens;
           if path.segments.len() > 0 && path.segments[0].ident == "builder" {
                   eprintln!("FOUND a builder attribute");
                   eprintln!("tokens {:?}",tokens);
                   let parsed = a.parse_meta();
                   eprintln!("Parsed {:#?}",parsed);

                   let meta = match parsed {
                       Ok(syn::Meta::List(syn::MetaList { path, nested, ..  } ))  => {
                           eprintln!("path  {:#?}",path);
                           eprintln!("path ident {:?}",path.segments[0].ident);
                           eprintln!("path  nested {:#?}",nested);
                           if nested.len() != 1 {
                               panic!("Only one builder option expected");
                            }
                           eprintln!("Nested first = {:#?}",nested.first().unwrap());
                           match nested.first() {
                               Some(syn::NestedMeta::Meta(syn::Meta::NameValue(syn::MetaNameValue {path, eq_token, lit } ))) => {
                                   eprintln!("Nested First decode {:?}",path);
                                   eprintln!("Nested First decode {:?}",eq_token);
                                   eprintln!("Nested First decode {:?}",lit);
                                   if path.segments[0].ident == "each" {
                                       eprintln!("Found Each");
                                    }
                                   if eq_token == token!([=]) {

                                       eprintln!("Found EQ");
                                    }

                                }
                               Some(x) => {
                                   eprintln!("Nested first Got unexpected {:?}",x);
                                }
                               
                               None => {
                                   eprintln!("None on nested.first");
                                }
                            }
                       },
                       Ok(other) => {
                           eprintln!("Got something unexpected");
                       },
                       Err(_) => {
                           eprintln!("Error on parse_meta");
                       },
                   };
            }
       }
       

        quote!{  #name: std::option::Option<#ty> }
    });

    // Builder default values
    let builder_init_fields = fields.iter().map(|f|
        {
           let name = &f.ident;
           quote!{  #name: None } 
       });

    // Builder Methods
    let builder_methods = fields.iter().map(|f|
        {
           let field_name = &f.ident;
           let field_type = match  unwrapped_option_type(&f.ty) {
               Some(updated) => updated,
               None => &f.ty,
            };
           quote!{  
                fn #field_name (&mut self, #field_name: #field_type) -> &mut Self {
                    self.#field_name = Some(#field_name);
                    self
                }
           }
       });


    let unset_fields = fields.iter().map(|f| {
       let field_name = &f.ident;
       let required_set = match  unwrapped_option_type(&f.ty) {
           Some(_) => false,
           None => true,
        };
       quote! {
           if #required_set && self.#field_name == None {
               Some(std::stringify!(#field_name).to_string())
           }
           else {
            None
            }
        }
    });

    // Output of build fields
    let output_fields = fields.iter().map(|f|
        {
           let field_name = &f.ident;
           let output = match  unwrapped_option_type(&f.ty) {
               Some(_) => quote! { #field_name : self.#field_name.clone() },
               None =>    quote! { #field_name : self.#field_name.clone().unwrap() },
            };
           output
       });

    //
    // OUTPUT
    let output : proc_macro::TokenStream = quote!( 
         pub struct #builder_name {
            #(#builder_def_fields,)*
         }
         
        impl #struct_name { 
            pub fn builder() -> #builder_name {
                #builder_name {
                    #(#builder_init_fields, )* 
                }
            }
        } 

        impl #builder_name {

            #(#builder_methods)*  

            fn build(&mut self) -> Result<#struct_name,  Box<dyn std::error::Error>> {

                let missing : Vec<String> = vec![ #(#unset_fields),* ].into_iter().filter_map(|e| e).collect();

                if missing.len() == 0 {
                    let x = #struct_name {
                        #(#output_fields),* ,
//                       #(#my_field_name:    #my_field_value ,)*
                    };

                    Ok(x)
                } 
                else {
                    let missing_list = missing.join(",");
                    let err = format!("The following fields are not yet set: {}",missing_list);
                    return std::result::Result::Err(err.into())

                }
            }

        }



        ).into();
    return output
}


