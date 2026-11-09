use proc_macro::TokenStream;
use quote::{ToTokens, quote};
use syn::{Data, DeriveInput, parse_macro_input};

#[proc_macro_derive(New)]
pub fn derive_new_constructor(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);

    let Data::Struct(data) = &input.data else {
        return syn::Error::new_spanned(&input, "this macro only supports structs")
            .to_compile_error()
            .into();
    };

    //will store the function params like name: type, name: type
    let mut parameters = quote! {};

    //will store names separed by comma like name, name , name
    let mut names = quote! {};

    for field in &data.fields {
        let indentifier = field.ident.as_ref().unwrap();
        names.extend(quote! {#indentifier,});
        let ty = &field.ty;

        parameters.extend(quote! {
            #indentifier: #ty,
        });
    }

    let name = input.ident;

    let (impl_genrics, ty_generics, where_clause) = input.generics.split_for_impl();

    let implementation = quote! {
        impl #impl_genrics #name #ty_generics #where_clause{
            pub fn new(#parameters)-> Self{
                Self{
                    #names
                }
            }
        }
    };
    implementation.into()
}
