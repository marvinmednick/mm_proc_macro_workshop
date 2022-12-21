use syn::{
    parse_macro_input, parse_quote, 
    Data, DeriveInput, 
    Error, Field, Fields, 
    FieldsNamed, DataStruct, GenericParam, Generics, 
    Lit,Meta,MetaNameValue,
};

#[proc_macro_derive(CustomDebug, attributes(debug))]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let _ = input;
    let parsed = parse_macro_input!(input as DeriveInput);
    let generics = parsed.generics;
    //eprintln!("Generics Before Adding Trait {:#?}\n ---------------------- End Before",generics);
    let generics = add_trait_bounds(generics);
    //eprintln!("Generics After adding Trait {:#?}\n-------- End After",generics);
     let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    //eprintln!("Impl Generics {:#?}",impl_generics);
    //eprintln!("Ty Generics {:#?}",ty_generics);
    //eprintln!("where Generics {:#?}",where_clause);

    let struct_name = parsed.ident.clone();
    //    eprintln!("Processing {:#?}",parsed);
    //
    // get the list of fields from the structure
    let fields = if let Data::Struct(
        DataStruct {
            fields: Fields::Named(FieldsNamed {
                ref named, ..
                }),
            ..
        }
    ) = parsed.data { named }
    else {
        // this dervive (builder) only supports structures at this time
        unimplemented!();
    };


    let field_info = fields.iter().map(|f| { 
        analyze_fields(&f)
    });


    let struct_name_string = format!("{}",struct_name);

    let output =  quote::quote!  {
        impl #impl_generics std::fmt::Debug for #struct_name #ty_generics #where_clause {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_struct(#struct_name_string)
                 #(#field_info)*
                 .finish()
            }
        }
    };
    return output.into();
}


fn analyze_fields (f: &Field) -> proc_macro2::TokenStream {

    fn mk_err<T: quote::ToTokens>(t: T) -> proc_macro2::TokenStream {
            Error::new_spanned(t, "expected `(debug = \"...\")`").to_compile_error()
    }

    let name = f.ident.clone().unwrap();
    let fld_ident = format!("{}",name);
    let attrs = &f.attrs;

    // check to see if there is a builder attributee
    if let std::option::Option::Some(a) = attrs.iter().find(|a| a.path.segments[0].ident == "debug") {

        let parsed = a.parse_meta();
//        eprintln!("Parsed is {:#?}",parsed);

         match parsed {
            std::result::Result::Ok(Meta::NameValue(MetaNameValue {path, eq_token: _ , lit : Lit::Str(ls) } )) => {
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


// Add a bound `T: HeapSize` to every type parameter T.
fn add_trait_bounds(mut generics: Generics) -> Generics {
    for param in &mut generics.params {
        if let GenericParam::Type(ref mut type_param) = *param {
            type_param.bounds.push(parse_quote!(std::fmt::Debug));
        }
    }
    generics
}

