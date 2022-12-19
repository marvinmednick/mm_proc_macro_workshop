
fn analyze_fields (f: &syn::Field) -> proc_macro2::TokenStream {

    fn mk_err<T: quote::ToTokens>(t: T) -> proc_macro2::TokenStream {
            syn::Error::new_spanned(t, "expected `(debug = \"...\")`").to_compile_error()
    }

    let name = f.ident.clone().unwrap();
    let fld_ident = format!("{}",name);
    let attrs = &f.attrs;

    // check to see if there is a builder attributee
    if let std::option::Option::Some(a) = attrs.iter().find(|a| a.path.segments[0].ident == "debug") {

        let parsed = a.parse_meta();
//        eprintln!("Parsed is {:#?}",parsed);

         match parsed {
            std::result::Result::Ok(syn::Meta::NameValue(syn::MetaNameValue {path, eq_token: _ , lit : syn::Lit::Str(ls) } )) => {
                if path.segments[0].ident == "debug" {

                    let fmt_string =  ls.value();
//                    eprintln!("Debug Format for this field is {:?}",fmt_string.as_bytes());

                    return quote::quote! {
                        .field(#fld_ident, &format_args!(#fmt_string,&self.#name))
                    };

                 }
                // Eq for MetaNameValue eq_token is ALWAYS Eq so no need to check
                else {
                    return quote::quote! {
                        .field(#fld_ident,&self.#name)
                    };
                }
            }
            std::result::Result::Ok(_other) => {
                eprintln!("Got something unexpected {:#?}",_other);
                return mk_err(a);
            },
            std::result::Result::Err(_) => {
                eprintln!("Error on parse_meta");
                return mk_err(a);
            },
        };
    }

 //   eprint!("Return 2");
    return quote::quote! {
        .field(#fld_ident, &self.#name)
    };

}

#[proc_macro_derive(CustomDebug, attributes(debug))]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let _ = input;
    let parsed = syn::parse_macro_input!(input as syn::DeriveInput);
    let struct_name = parsed.ident.clone();
    //    eprintln!("Processing {:#?}",parsed);
    //
    // get the list of fields from the structure
    let fields = if let syn::Data::Struct(
        syn::DataStruct {
            fields: syn::Fields::Named(syn::FieldsNamed {
                ref named, ..
                }),
            ..
        }
    ) = parsed.data { named }
    else {
        // this dervive (builder) only supports structures at this time
        unimplemented!();
    };


    /*
    eprintln!("Struct name {}",struct_name);
    for f in fields.clone() {
        let name = f.ident.clone().unwrap();
        //eprintln!("Field Name: {} {:#?}",name, f);

    }
    */


    let field_info = fields.iter().map(|f| { 
//        let name = f.ident.clone().unwrap();
//        let fld_ident = format!("{}",name);
//        quote::quote! {
//            .field(#fld_ident,&self.#name)
 //       }
        analyze_fields(&f)
    });


    let struct_name_string = format!("{}",struct_name);

    let output =  quote::quote!  {
        impl std::fmt::Debug for Field {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_struct(#struct_name_string)
                 #(#field_info)*
                 .finish()
            }
        }
    };
    return output.into();
}
