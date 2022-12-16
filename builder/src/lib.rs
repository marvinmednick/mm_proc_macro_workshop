
fn unwrapped_option_type<'a>(ty : &'a syn::Type) -> std::option::Option<&'a syn::Type> {

    // check that path is of a type 
    
    if let syn::Type::Path(type_path) = ty {

        // default return to None
        // get the last segment
        if let std::option::Option::Some(seg) = type_path.path.segments.last() {
            // check if its not
            if seg.ident == "Option" {
                if let syn::PathArguments::AngleBracketed(
                    syn::AngleBracketedGenericArguments {
                        ref args,
                        ..
                    }
                ) = seg.arguments {
                    if let std::option::Option::Some(syn::GenericArgument::Type(inner_type)) = args.first() {
                        return std::option::Option::Some(inner_type)
                    }
                }

            }
        }

    }
    // default to None if doesn't match
    return std::option::Option::None
}



fn is_vec(ty : &syn::Type )  -> Option<&syn::Type> {

    if let syn::Type::Path( syn::TypePath { qself: _, path: syn::Path { leading_colon :  _, segments } }) = ty {
        if segments.last().unwrap().ident == "Vec"  {
            if let syn::PathArguments::AngleBracketed( syn::AngleBracketedGenericArguments { colon2_token: _ , lt_token: _, args, ..}) = &segments.last().unwrap().arguments { 
                    if let  syn::GenericArgument::Type(inner_type) = args.last().unwrap() {
                        //eprintln!("vec type is {:#?}",inner_type);
                        return Some(inner_type);
                    }
                
            }
        }
    }
    return std::option::Option::None;
}

struct FieldBuilderMetadata {
    name:  syn::Ident,
    optional: bool,
    inner_type:  syn::Type,
    set_field_code: Option<proc_macro2::TokenStream>,
    can_set_each: bool,
}

fn analyze_fields (f: &syn::Field) -> Option<FieldBuilderMetadata> {

    fn mk_err<T: quote::ToTokens>(t: T) -> Option<proc_macro2::TokenStream> {
        std::option::Option::Some(
            syn::Error::new_spanned(t, "expected `builder(each = \"...\")`").to_compile_error(),
        )
    }


    let name = f.ident.clone().unwrap();
    let attrs = &f.attrs;
    let (ty, optional) = match  unwrapped_option_type(&f.ty) {
       std::option::Option::Some(updated) => (updated, true),
       std::option::Option::None => (&f.ty, false),
    }.clone();

    let mut  field_info = FieldBuilderMetadata {
        name: name.clone(),
        optional,
        inner_type: ty.clone(),
        set_field_code: std::option::Option::None,
        can_set_each: false,
    };

    // check to see if there is a builder attributee
    if let std::option::Option::Some(a) = attrs.iter().find(|a| a.path.segments[0].ident == "builder") {

        let parsed = a.parse_meta();
         match parsed {
            std::result::Result::Ok(syn::Meta::List(nvs))  => {
                let nested = nvs.nested.clone();
                if nested.len() != 1 {
                    panic!("Only one builder option expected");
                }
                match nested.first() {
                    std::option::Option::Some(syn::NestedMeta::Meta(syn::Meta::NameValue(syn::MetaNameValue {path, eq_token: _ , lit : syn::Lit::Str(ls) } ))) => {
                        if path.segments[0].ident == "each" {
                            // check to see if source is a vector

                            let inner_ty = match is_vec(&ty) {
                                std::option::Option::Some(ty) => ty,
                                std::option::Option::None => {
                                    field_info.set_field_code = mk_err(nested);
                                    return std::option::Option::Some(field_info);
                                }
                            };


                            let ls_id =  quote::format_ident!("{}",ls.value());
                            let add_set_function = quote::quote! {
                                fn #ls_id (&mut self, #ls_id: #inner_ty) -> &mut Self {
                                    self.#name.push(#ls_id);
                                    self
                                }
                            };
                            let full_set_function = quote::quote!{  
                                fn #name (&mut self, #name: #ty) -> &mut Self {
                                    self.#name = #name;
                                    self
                                }
                            };

                            field_info.can_set_each = true;

                            // check to see if the name configured for the each attribute is the same as the original (which indicates that we can't have
                            // both since their specified to have the same name, but different parameters. Since its not specified in the test description,
                            // we're goin to assume that the desire is that there is only one function and it adds an additional item to the vector
                            if name == ls_id {
                           //     eprintln!("analyze:  Names match need to only output a single function named {}",ls_id);
                                // in this case, we want to generate 1 set function. Set function must initialize vec if not already set
                                // Init function can still be none could or could not be optional to set   (assume it is for
                                // now)  -- note if not optional default should be set to 
                                field_info.set_field_code = std::option::Option::Some(add_set_function);
                                return std::option::Option::Some(field_info);
                            }
                            else {
                           //     eprintln!("analyze:  Names DONT match output vector function {} and {}",name, ls_id);
                                // in this case, we want to generate 2 set function.
                                // Init function can still set to None
                                // could or could not be optional to set  (

                                field_info.set_field_code = std::option::Option::Some(quote::quote! {
                                        #add_set_function
                                        #full_set_function
                                });
                                return std::option::Option::Some(field_info);
                             }
                         }
                        // Eq for MetaNameValue eq_token is ALWAYS Eq so no need to check
                        else {
                            eprintln!("Unknown builder attribute {}",name);
                            field_info.set_field_code = mk_err(nvs);
                            return std::option::Option::Some(field_info);
                        }
                     }
                    std::option::Option::Some(x) => {
                        eprintln!("Nested first Got unexpected {:?}",x);
                        field_info.set_field_code = mk_err(x);
                        return std::option::Option::Some(field_info);
                     }
                    
                    std::option::Option::None => {
                        eprintln!("None on nested.first");
                        field_info.set_field_code = mk_err(a);
                        return std::option::Option::Some(field_info);
                     }
                 }
            },
            std::result::Result::Ok(_other) => {
                eprintln!("Got something unexpected");
                field_info.set_field_code = mk_err(a);
                return std::option::Option::Some(field_info);
            },
            std::result::Result::Err(_) => {
                eprintln!("Error on parse_meta");
                field_info.set_field_code = mk_err(a);
                return std::option::Option::Some(field_info);
            },
        };
    }

    let full_set_function = quote::quote!{  
        fn #name (&mut self, #name: #ty) -> &mut Self {
            self.#name = std::option::Option::Some(#name);
            self
        }
    };

    field_info.set_field_code = std::option::Option::Some(full_set_function);
    return std::option::Option::Some(field_info);

}

#[proc_macro_derive(Builder, attributes(builder))]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input3 = input.clone();

    let parsed_input : syn::DeriveInput = syn::parse(input3).unwrap();
    let parsed_copy = parsed_input.clone();

    let struct_name = parsed_input.ident;
    let builder_name = quote::format_ident!("{}Builder",struct_name);


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

    let field_metadata : std::vec::Vec<FieldBuilderMetadata>= fields.iter().map(|f| analyze_fields(f).unwrap()).collect();

    //////////////////////////////////////////////////////////
    // builder structure fields
    let builder_definition_data : std::vec::Vec<_> = field_metadata.iter().map(|f| 
        (f.name.clone(),f.inner_type.clone(),f.can_set_each.clone())).collect();

    let builder_definition : std::vec::Vec<_> = builder_definition_data.iter().map(|(name,inner_type,can_set_each) |  {
        if *can_set_each {
                quote::quote! { #name : #inner_type }
        } 
        else {
            quote::quote! { #name : std::option::Option<#inner_type> }
        }
    }).collect();


    //////////////////////////////////////////////////////////
    // Builder default values
    let names : std::vec::Vec<_> = field_metadata.iter().map(|f| (f.name.clone(), f.can_set_each)).collect();

    let builder_init_fields = names.iter().map(|(name, can_set_each)|
        if *can_set_each {
           quote::quote!{  #name: vec![] } 
        }
        else {
           quote::quote!{  #name: std::option::Option::None } 
       });

    //////////////////////////////////////////////////////////
    // Builder Methods
    let set_field_funcs : std::vec::Vec<_> = field_metadata.iter().map(|f| f.set_field_code.clone()).collect();

    let builder_methods : std::vec::Vec<_> = set_field_funcs.iter().map(|set_func| 
         quote::quote!{  
                #set_func
           }
       ).collect();


    //////////////////////////////////////////////////////////
    // unset field checks Methods
    let optional : std::vec::Vec<_> = field_metadata.iter().map(|f| (f.name.clone(),f.optional.clone(),f.can_set_each.clone())).collect();

    let unset_fields = optional.iter().map(|(name, is_optional, can_set_each)| 
        if  *is_optional || *can_set_each {
            quote::quote! { 
                std::option::Option::None
            }
        } 
        else {
           quote::quote! {
               if self.#name == std::option::Option::None {
                   std::option::Option::Some(std::stringify!(#name).to_string())
               }
               else {
                    std::option::Option::None
               }
            }
        }
    );

    //////////////////////////////////////////////////////////
    // Output of build fields

    let output_fields : std::vec::Vec <_> = optional.iter().map(|(name,is_optional,can_set_each)| 
        if ! is_optional {
            if *can_set_each {
                // Not setup as Optional -- empty fields will be an empty vec not as None
               quote::quote! { #name : self.#name.clone() }
            }
            else {
               quote::quote! { #name : self.#name.clone().unwrap() }
            }
        }
        else {
               quote::quote! { #name : self.#name.clone() }
        }
    ).collect();


    //
    // OUTPUT
    let output : proc_macro::TokenStream = quote::quote!( 
         pub struct #builder_name {
            #(#builder_definition,)*
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

            fn build(&mut self) -> std::result::Result<#struct_name,  std::boxed::Box<dyn std::error::Error>> {

                let missing : std::vec::Vec<String> = vec![ #(#unset_fields),* ].into_iter().filter_map(|e| e).collect();

                if missing.len() == 0 {
                    let x = #struct_name {
                        #(#output_fields),* ,
                    };

                    std::result::Result::Ok(x)
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


