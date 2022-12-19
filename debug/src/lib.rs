use proc_macro::TokenStream;

#[proc_macro_derive(CustomDebug)]
pub fn derive(input: TokenStream) -> TokenStream {
    let _ = input;
    eprintln!("Token stream {:#?}",input);

    return quote::quote! {}.into();
}
