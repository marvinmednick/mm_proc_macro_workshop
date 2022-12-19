
#[proc_macro_derive(CustomDebug)]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let _ = input;
    let parsed = syn::parse_macro_input!(input as syn::DeriveInput);
    let struct_name = parsed.ident;
    //eprintln!("Processing {:#?}",struct_name);
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
        eprintln!("Field Name: {}",name);
    }
    */

    let field_info = fields.iter().map(|f| { 
        let fld_name = f.ident.clone().unwrap();
        let fld_ident = format!("{}",fld_name);

        quote::quote! {
           .field(#fld_ident, &self.#fld_name)
        }
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
