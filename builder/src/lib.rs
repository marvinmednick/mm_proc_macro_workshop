use proc_macro::TokenStream;
use quote::{quote,format_ident};
use syn::{DeriveInput,Path};

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

/*
enum SetFunctionConfig {
    Set_Individual,
    Set_All,
    Both
}

struct FieldBuildInfo {
    set_config: SetFunctionConfig,
    name : syn::Ident,
    ty: syn::Type,

}
*/

fn is_vec(ty : &syn::Type )  -> Option<&syn::Type> {

    if let syn::Type::Path( syn::TypePath { qself: _, path: syn::Path { leading_colon :  _, segments } }) = ty {
        if segments.last().unwrap().ident == "Vec"  {
            if let syn::PathArguments::AngleBracketed( syn::AngleBracketedGenericArguments { colon2_token: _ , lt_token: _, args, ..}) = &segments.last().unwrap().arguments { 
                    if let  syn::GenericArgument::Type(inner_type) = args.last().unwrap() {
                        eprintln!("vec type is {:#?}",inner_type);
                        return Some(inner_type);
                    }
                
            }
        }
    }
    return None;
}

fn analyze_fields (f: &syn::Field) -> Option<proc_macro2::TokenStream> {

    fn mk_err<T: quote::ToTokens>(t: T) -> Option<proc_macro2::TokenStream> {
        Some(
            syn::Error::new_spanned(t, "expected `builder(each = \"...\")`").to_compile_error(),
        )
    }

    let name = f.ident.clone().unwrap();
    let attrs = &f.attrs;
    let ty = match  unwrapped_option_type(&f.ty) {
       Some(updated) => updated,
       None => &f.ty,
    }.clone();

//    let fn_name = format_ident!("alt_{}",name);

    let full_set_function = quote!{  
        fn #name (&mut self, #name: #ty) -> &mut Self {
            self.#name = Some(#name);
            self
        }
    };

//    let set_type = SetFunctionConfig::Set_All;

    // check to see if there is a builder attributee
    if let Some(a) = attrs.iter().find(|a| a.path.segments[0].ident == "builder") {

        match a.parse_meta() {
            Ok(syn::Meta::List(syn::MetaList { path: _, nested, ..  } ))  => {
                if nested.len() != 1 {
                    panic!("Only one builder option expected");
                }
                match nested.first() {
                    Some(syn::NestedMeta::Meta(syn::Meta::NameValue(syn::MetaNameValue {path, eq_token: _ , lit : syn::Lit::Str(ls) } ))) => {
                        if path.segments[0].ident == "each" {
                            // check to see if source is a vector

                            let inner_ty = match is_vec(&ty) {
                                Some(ty) => ty,
                                None => return mk_err(&f.ty),
                            };


                            let ls_id =  format_ident!("{}",ls.value());
                            let add_set_function = quote! {
                                fn #ls_id (&mut self, #ls_id: #inner_ty) -> &mut Self {
                                    self.#name.push(#ls_id);
                                    self
                                }
                            };

                            // check to see if the name configured for the each attribute is the same as the original (which indicates that we can't have
                            // both since their specified to have the same name, but different parameters. Since its not specified in the test description,
                            // we're goin to assume that the desire is that there is only one function and it adds an additional item to the vector
                            if name == ls_id {
                                eprintln!("analyze:  Names match need to only output a single function named {}",ls_id);
                                // in this case, we want to generate 1 set function. Set function must initialize vec if not already set
                                // Init function can still be none could or could not be optional to set   (assume it is for
                                // now)  -- note if not optional default should be set to 
                                return Some(add_set_function);
                            }
                            else {
                                eprintln!("analyze:  Names DONT match output vector function {} and {}",name, ls_id);
                                // in this case, we want to generate 2 set function.
                                // Init function can still set to None
                                // could or could not be optional to set  (

                                return Some(quote! {
                                    #full_set_function
                                    #add_set_function
                                });
                             }
                         }
                        // Eq for MetaNameValue eq_token is ALWAYS Eq so no need to check
                        else {
                            return mk_err(ty);
                        }
                     }
                    Some(x) => {
                        eprintln!("Nested first Got unexpected {:?}",x);
                        return mk_err(x);
                     }
                    
                    None => {
                        eprintln!("None on nested.first");
                        return mk_err(a);
                     }
                 }
            },
            Ok(_other) => {
                eprintln!("Got something unexpected");
                return mk_err(f);
            },
            Err(_) => {
                eprintln!("Error on parse_meta");
                return mk_err(f);
            },
        };
    }

    return Some(full_set_function);

}

#[proc_macro_derive(Builder, attributes(builder))]
pub fn derive(input: TokenStream) -> TokenStream {
    let input3 = input.clone();

    let parsed_input : DeriveInput = syn::parse(input3).unwrap();
    let parsed_copy = parsed_input.clone();

    let struct_name = parsed_input.ident;
    let builder_name = format_ident!("{}Builder",struct_name);


    // get the list of fields from the structure
    let fields = if let syn::Data::Struct(
        syn::DataStruct {
            fields: syn::Fields::Named(syn::FieldsNamed {
                ref named, ..
                }),
            ..
        }
    ) = parsed_copy.data { named }
    else {
        // this dervive (builder) only supports structures at this time
        unimplemented!();
    };

    //////////////////////////////////////////////////////////
    // builder structure fields
    let builder_def_fields = fields.iter().map(|f| {

        // process each field f
       let name = &f.ident.clone().unwrap();
       let ty = match  unwrapped_option_type(&f.ty) {
           Some(updated) => updated,
           None => &f.ty,
        };

        quote!{  #name: std::option::Option<#ty> }
    });

    //////////////////////////////////////////////////////////
    // Builder default values
    let builder_init_fields = fields.iter().map(|f|
        {
           let name = &f.ident;
           quote!{  #name: None } 
       });

    //////////////////////////////////////////////////////////
    // Builder Methods
    let builder_methods = fields.iter().map(|f|
        {
           let set_func_fields =  analyze_fields(f).unwrap();

           let field_name = &f.ident;
           let field_type = match  unwrapped_option_type(&f.ty) {
               Some(updated) => updated,
               None => &f.ty,
            };

           quote!{  
                #set_func_fields
           }
       });


    //////////////////////////////////////////////////////////
    // unset field checks Methods
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

    //////////////////////////////////////////////////////////
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


