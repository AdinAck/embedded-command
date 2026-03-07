pub(crate) mod vanilla;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{DeriveInput, Generics, Ident, ItemType, Path};

#[derive(Clone)]
struct BodyInfo {
    ident: Ident,
    generics: Generics,
    path: Path,
}

pub fn impl_serialize_buf(item: TokenStream) -> TokenStream {
    let item: DeriveInput = syn::parse2(item.into()).unwrap();

    if !item.generics.params.is_empty() {
        panic!("SerializeBuf is incompatible with generic types. You may still use SerializeIter.");
    }

    let info = BodyInfo {
        ident: item.ident,
        generics: item.generics,
        path: syn::parse2(quote! { serac }).unwrap(),
    };

    let path = info.path;
    let ident = info.ident;

    quote! {
        unsafe impl #path::SerializeBuf<{ <#ident as #path::Size>::SIZE }> for #ident {}
    }
    .into()
}

pub fn impl_serialize_buf_alias(attrs: TokenStream, item: TokenStream) -> TokenStream {
    let item = item.into();
    let attrs: TokenStream2 = attrs.into();

    let original = quote! { #attrs #item };

    let item: ItemType = syn::parse2(item).expect("a");

    let path = syn::parse2::<Path>(quote! { serac }).expect("b");
    let ident = item.ident;

    quote! {
        #original

        unsafe impl #path::SerializeBuf<{ <#ident as #path::Size>::SIZE }> for #ident {}
    }
    .into()
}
