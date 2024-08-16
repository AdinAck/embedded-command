use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{
    Attribute, Data, DataEnum, DataStruct, DeriveInput, Fields, Generics, Ident, Index, Path, Type,
    Variant,
};

#[derive(Clone)]
struct BodyInfo {
    ident: Ident,
    generics: Generics,
    path: Path,
}

fn get_repr<'a>(mut attrs: impl Iterator<Item = &'a Attribute>) -> Type {
    attrs
        .find(|&attr| attr.path().is_ident("repr"))
        .expect("Enum must have #[repr(...)] attribute.")
        .parse_args()
        .expect("#[repr(...) can only have one type.")
}

fn build_tags<'a>(variants: impl Iterator<Item = &'a &'a Variant>) -> Vec<TokenStream2> {
    let mut tags = Vec::new();
    let mut i = 0; // count up by one starting at any known tag
    let mut last_anchor = quote! { 0 };

    for variant in variants {
        if let Some((_, tag)) = &variant.discriminant {
            // a tag is provided, restart counter and update as last anchor
            let tokens = quote! { #tag };
            tags.push(tokens.clone());
            i = 0;
            last_anchor = tokens;
        } else {
            // a tag was not explicitly provided, we need to count up from last anchor
            let rendered_offset = Index::from(i);
            tags.push(quote! { #last_anchor + #rendered_offset });
        }
        i += 1;
    }

    tags
}

fn serialize_struct(s: DataStruct, info: &BodyInfo) -> TokenStream2 {
    let implementer = &info.ident;
    let path = &info.path;
    let (impl_generics, ty_generics, where_clause) = info.generics.split_for_impl();

    let types: Vec<_> = s.fields.iter().map(|field| &field.ty).collect();

    let (ser_body, deser_body) = match &s.fields {
        Fields::Unit => (quote! { Ok(()) }, quote! { Ok(Self) }),
        Fields::Unnamed(fields) => {
            let attr_tags: Vec<_> = fields
                .unnamed
                .iter()
                .enumerate()
                .map(|(i, _)| Index::from(i))
                .collect();

            (
                quote! {
                    let mut dst = dst.into_iter();

                    #(
                        #path::SerializeIter::serialize_iter(&self.#attr_tags, &mut dst)?;
                    )*

                    Ok(())
                },
                quote! {
                    let mut src = src.into_iter();

                    Ok(
                        Self(
                            #(
                                <#types as #path::SerializeIter>::deserialize_iter(&mut src)?,
                            )*
                        )
                    )
                },
            )
        }
        Fields::Named(fields) => {
            let attr_idents: Vec<_> = fields
                .named
                .iter()
                .map(|field| field.ident.as_ref().unwrap())
                .collect();

            (
                quote! {
                    let mut dst = dst.into_iter();

                    #(
                        #path::SerializeIter::serialize_iter(&self.#attr_idents, &mut dst)?;
                    )*

                    Ok(())
                },
                quote! {
                    let mut src = src.into_iter();

                    Ok(
                        Self {
                            #(
                                #attr_idents: <#types as #path::SerializeIter>::deserialize_iter(&mut src)?,
                            )*
                        }
                    )
                },
            )
        }
    };

    quote! {
        impl #impl_generics #path::SerializeIter for #implementer #ty_generics #where_clause {
            fn serialize_iter<'a>(&self, dst: impl IntoIterator<Item = &'a mut <#path::encoding::vanilla::Vanilla as #path::encoding::Encoding>::Word>) -> Result<(), #path::error::EndOfInput>
            where
                <#path::encoding::vanilla::Vanilla as #path::encoding::Encoding>::Word: 'a,
            {
                #ser_body
            }

            fn deserialize_iter<'a>(src: impl IntoIterator<Item = &'a <#path::encoding::vanilla::Vanilla as #path::encoding::Encoding>::Word>) -> Result<Self, #path::error::Error>
            where
                <#path::encoding::vanilla::Vanilla as #path::encoding::Encoding>::Word: 'a,
            {
                #deser_body
            }
        }
    }
}

fn size_of_struct(s: DataStruct, info: &BodyInfo) -> TokenStream2 {
    let types: Vec<_> = s.fields.iter().map(|field| &field.ty).collect();
    let path = &info.path;

    if types.is_empty() {
        quote! { 0 }
    } else {
        quote! { #( <<#types as #path::SerializeBuf>::Serialized as #path::medium::Medium>::SIZE )+* }
    }
}

fn serialize_enum(e: DataEnum, info: &BodyInfo, repr: Type) -> TokenStream2 {
    let implementer = &info.ident;
    let path = &info.path;
    let (impl_generics, ty_generics, where_clause) = info.generics.split_for_impl();
    let variants: Vec<_> = e.variants.iter().collect();

    let tags: Vec<_> = build_tags(variants.iter());
    let tag_consts: Vec<_> = variants
        .iter()
        .map(|variant| {
            let ident = &variant.ident;
            format_ident!(
                "{}_TAG",
                inflector::cases::screamingsnakecase::to_screaming_snake_case(&ident.to_string())
            )
        })
        .collect();

    let ser_arms: Vec<_> = variants
        .iter()
        .zip(tag_consts.iter())
        .map(|(variant, tag_const)| {
            let ident = &variant.ident;
            match &variant.fields {
                Fields::Unit => quote! {
                    #ident => {
                        #path::SerializeIter::serialize_iter(&#tag_const, &mut dst)
                    }
                },
                Fields::Unnamed(fields) => {
                    let idents: Vec<_> = fields
                        .unnamed
                        .iter()
                        .enumerate()
                        .map(|(i, _field)| {
                            let ident = format_ident!("v{i}");

                            quote! { #ident }
                        })
                        .collect();

                    quote! {
                        #ident(#(#idents),*) => {
                            #path::SerializeIter::serialize_iter(&#tag_const, &mut dst)?;
                            #(
                                #path::SerializeIter::serialize_iter(#idents, &mut dst)?;
                            )*

                            Ok(())
                        }
                    }
                }
                Fields::Named(fields) => {
                    let idents: Vec<_> = fields
                        .named
                        .iter()
                        .map(|field| field.ident.as_ref().unwrap())
                        .collect();

                    quote! {
                        #ident{#(#idents),*} => {
                            #path::SerializeIter::serialize_iter(&#tag_const, &mut dst)?;
                            #(
                                #path::SerializeIter::serialize_iter(#idents, &mut dst)?;
                            )*

                            Ok(())
                        }
                    }
                }
            }
        })
        .collect();

    let deser_arms: Vec<_> = variants
        .iter()
        .map(|variant| {
            let ident = &variant.ident;
            match &variant.fields {
                Fields::Unit => quote! {
                    #ident
                },
                Fields::Unnamed(fields) => {
                    let types: Vec<_> = fields.unnamed.iter().map(|field| &field.ty).collect();
                    quote! {
                        #ident (
                            #(
                                <#types as #path::SerializeIter>::deserialize_iter(&mut src)?,
                            )*
                        )
                    }
                }
                Fields::Named(fields) => {
                    let idents: Vec<_> = fields
                        .named
                        .iter()
                        .map(|field| field.ident.as_ref().unwrap())
                        .collect();
                    let types: Vec<_> = fields.named.iter().map(|field| &field.ty).collect();

                    quote! {
                        #ident {
                            #(
                                #idents: <#types as #path::SerializeIter>::deserialize_iter(&mut src)?,
                            )*
                        }
                    }
                }
            }
        })
        .collect();

    quote! {
        impl #impl_generics #path::SerializeIter for #implementer #ty_generics #where_clause {
            fn serialize_iter<'a>(&self, dst: impl IntoIterator<Item = &'a mut <#path::encoding::vanilla::Vanilla as #path::encoding::Encoding>::Word>) -> Result<(), #path::error::EndOfInput>
            where
                <#path::encoding::vanilla::Vanilla as #path::encoding::Encoding>::Word: 'a,
            {
                let mut dst = dst.into_iter();

                #(
                    const #tag_consts: #repr = #tags;
                )*

                match self {
                    #(
                        Self::#ser_arms,
                    )*
                }
            }

            fn deserialize_iter<'a>(src: impl IntoIterator<Item = &'a <#path::encoding::vanilla::Vanilla as #path::encoding::Encoding>::Word>) -> Result<Self, #path::error::Error>
            where
                <#path::encoding::vanilla::Vanilla as #path::encoding::Encoding>::Word: 'a,
            {
                let mut src = src.into_iter();

                #(
                    const #tag_consts: #repr = #tags;
                )*

                let tag = <#repr as #path::SerializeIter>::deserialize_iter(&mut src)?;

                match tag {
                    #(
                        #tag_consts => Ok(Self::#deser_arms),
                    )*
                    _ => Err(#path::error::Error::Invalid)
                }
            }
        }
    }
}

fn size_of_enum(e: DataEnum, info: &BodyInfo, repr: Type) -> TokenStream2 {
    let path = &info.path;
    let sizes: Vec<_> = e
        .variants
        .iter()
        .filter_map(|variant| {
            if !variant.fields.is_empty() {
                let types: Vec<_> = variant
                    .fields
                    .iter()
                    .map(|field| &field.ty)
                    .collect();

                Some(quote! { #(<<#types as #path::SerializeBuf>::Serialized as #path::medium::Medium>::SIZE)+* })
            } else {
                None
            }
        })
        .collect();

    quote! {{
        let mut max = 0;

        #(
            if #sizes > max {
                max = #sizes;
            }
        )*

        max + <<#repr as #path::SerializeBuf>::Serialized as #path::medium::Medium>::SIZE
    }}
}

pub fn serialize_iter(item: TokenStream) -> TokenStream {
    let item: DeriveInput = syn::parse2(item.into()).unwrap();

    let info = BodyInfo {
        ident: item.ident,
        generics: item.generics,
        path: syn::parse2(quote! { cookie_cutter }).unwrap(),
    };

    let implementation = match item.data {
        Data::Struct(s) => serialize_struct(s, &info),
        Data::Enum(e) => serialize_enum(e, &info, get_repr(item.attrs.iter())),
        _ => panic!("Vanilla serializer is only implemented for structs and enums."),
    };

    implementation.into()
}

pub fn serialize_buf(item: TokenStream) -> TokenStream {
    let item: DeriveInput = syn::parse2(item.into()).unwrap();

    if !item.generics.params.is_empty() {
        panic!("SerializeBuf is incompatible with generic types. You may still use SerializeIter.");
    }

    let info = BodyInfo {
        ident: item.ident,
        generics: item.generics,
        path: syn::parse2(quote! { cookie_cutter }).unwrap(),
    };

    let size = match item.data {
        Data::Struct(s) => size_of_struct(s, &info),
        Data::Enum(e) => size_of_enum(e, &info, get_repr(item.attrs.iter())),
        _ => panic!("Vanilla serializer is only implemented for structs and enums."),
    };

    let path = info.path;
    let ident = info.ident;
    let (impl_generics, ty_generics, where_clause) = info.generics.split_for_impl();
    let ty = quote! { #ident #ty_generics };

    quote! {
        unsafe impl #impl_generics #path::SerializeBuf for #ty #ty_generics #where_clause {
            type Serialized = [u8; #size];
        }
    }
    .into()
}
