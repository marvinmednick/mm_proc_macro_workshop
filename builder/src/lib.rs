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


enum SetFunctionConfig {
    Individual,
    Global,
    Both
}

struct FieldBuildInfo {
    optional: bool,
    set_config: SetFunctionConfig,
    name : syn::Ident,
    ty: syn::Type,

}

fn is_vec(ty : &syn::Type )  -> bool {

    if let syn::Type::Path( syn::TypePath { qself: _, path: syn::Path { leading_colon :  _, segments } }) = ty {
        if segments.last().unwrap().ident == "Vec"  && 
        if let syn::PathArguments::AngleBracketed(..) = segments.last().unwrap().arguments { true } else {false} {
                return true
        }
    } 
    return false

}

fn anaylze_fields (f: &syn::Field) -> FieldBuildInfo {

    let name = f.ident.clone().unwrap();
    let ty = match  unwrapped_option_type(&f.ty) {
       Some(updated) => updated,
       None => &f.ty,
    }.clone();


    let  output = if let syn::Field {attrs, ..} = f {
        
        // check to see if there is a builder attributee
        if let Some(a) = attrs.iter().find(|a| a.path.segments[0].ident == "builder") {
            eprintln!("DEFINE -- for field {:?}  found builder attribute", name);

            //TODO -- check to verify field is a Vector -- each doesn't make sense unless the field
            //is a Vec
 
            let output = match a.parse_meta() {
                Ok(syn::Meta::List(syn::MetaList { path, nested, ..  } ))  => {
                    if nested.len() != 1 {
                        panic!("Only one builder option expected");
                    }
                    match nested.first() {
                        Some(syn::NestedMeta::Meta(syn::Meta::NameValue(syn::MetaNameValue {path, eq_token, lit : syn::Lit::Str(ls) } ))) => {
                            if path.segments[0].ident == "each" {

                                // check to see if the name configured for the each attribute is
                                // the same as the original (which indicates that we can't have
                                // both since their specified to have the same name, but different
                                // parameters. Since its not specified in the test description,
                                // we're goin to assume that the desire is that there is only one
                                // function and it adds an additional item to the vector
                                let ls_id =  format_ident!("{}",ls.value());
                                if name == ls_id {
                                    eprintln!("Names match need to only output a single function named {}",ls_id);
                                    // in this case, we want to generate 1 set function.
                                    // Set function must initialize vec if not already set
                                    // Init function can still be none
                                    // could or could not be optional to set   (assume it is for
                                    // now)  -- note if not optional default should be set to 

                                    FieldBuildInfo { optional: true, set_config: SetFunctionConfig::Individual, name, ty }
                                 }
                                else {
                                    eprintln!("Names DONT match output vector function {} and {}",name, ls_id);
                                    // in this case, we want to generate 2 set function.
                                    // Init function can still set to None
                                    // could or could not be optional to set  (
                                    FieldBuildInfo { optional: true, set_config: SetFunctionConfig::Both,  name, ty } 
                                 }
                             }
                            // Eq for MetaNameValue eq_token is ALWAYS Eq so no need to
                            // check
                            
                            else {
                                    FieldBuildInfo { optional: true, set_config: SetFunctionConfig::Global,  name, ty } 
                            }
                         }
                        Some(x) => {
                            eprintln!("Nested first Got unexpected {:?}",x);
                                    FieldBuildInfo { optional: true, set_config: SetFunctionConfig::Individual, name, ty }
                         }
                        
                        None => {
                            eprintln!("None on nested.first");
                                    FieldBuildInfo { optional: true, set_config: SetFunctionConfig::Individual, name, ty }
                         }
                     }
                },
                Ok(other) => {
                    eprintln!("Got something unexpected");
                                    FieldBuildInfo { optional: true, set_config: SetFunctionConfig::Individual, name, ty }
                },
                Err(_) => {
                    eprintln!("Error on parse_meta");
                                    FieldBuildInfo { optional: true, set_config: SetFunctionConfig::Individual, name, ty }
                },
            };
            output
        }
        else {
            eprintln!("DEFINE -- for field {:?} NO builder attribute", name);
            // did not get a builder attribute so 
            // standard setter as previously defined
            FieldBuildInfo { optional: true, set_config: SetFunctionConfig::Global,  name, ty } 
        }

    } 
    else {
        FieldBuildInfo { optional: true, set_config: SetFunctionConfig::Global,  name, ty } 
    };
    output

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

        anaylze_fields(f);

        // process each field f
       let name = &f.ident.clone().unwrap();
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
   //                eprintln!("tokens {:?}",tokens);
                   let parsed = a.parse_meta();
   //                eprintln!("Parsed {:#?}",parsed);

                   let meta = match parsed {
                       Ok(syn::Meta::List(syn::MetaList { path, nested, ..  } ))  => {
//                           eprintln!("path  {:#?}",path);
 //                          eprintln!("path ident {:?}",path.segments[0].ident);
  //                         eprintln!("path  nested {:#?}",nested);
                           if nested.len() != 1 {
                               panic!("Only one builder option expected");
                            }
//                         eprintln!("Nested first = {:#?}",nested.first().unwrap());
                           match nested.first() {
                               Some(syn::NestedMeta::Meta(syn::Meta::NameValue(syn::MetaNameValue {path, eq_token, lit : syn::Lit::Str(ls) } ))) => {
                                   if path.segments[0].ident == "each" {
                                       eprintln!("Found Each");
                                       eprintln!("Nested First ls {:?}",ls);
                                       let ls_id =  format_ident!("{}",ls.value());
                                       if name == &ls_id {
                                           eprintln!("Names match need to only output a single function named {}",ls_id);
                                        }
                                       else {
                                           eprintln!("Names DONT match output vector function {} and {}",name, ls_id);
                                        }
                                    }
                                   // Eq for MetaNameValue eq_token is ALWAYS Eq so no need to
                                   // check
                                   
/*
                                   if let syn::Lit::Str (ls) = lit {

                                        let lp = ls.value();
//                                       eprintln!("Nested First lit {:?}",lit);
                                       eprintln!("Nested First ls {:?}",ls);
                                       eprintln!(" lit parse {:#?}",lp);
                                       let l_id =  format_ident!("{}",lp);
                                       eprintln!(" lit parse {:#?} {}",lp,l_id);
                                       // at this point there are two options
                                       // if the each version name is the same as field name,
                                       // output only the individual one
                                       // if the each version name is different output both the
                                       // originand the each version
//                                       let indiv_method = syn::Ident::new(l_id,ls.span())

                                   }
*/

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


