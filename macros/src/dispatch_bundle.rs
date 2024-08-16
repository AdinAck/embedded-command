use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{
    punctuated::Punctuated, token::Paren, Field, FieldMutability, Fields, FieldsUnnamed, Ident,
    ItemEnum, Type, Visibility,
};

pub fn bundle(attr: TokenStream, item: TokenStream) -> TokenStream {
    // capture trait ident and enum body
    let tr: Ident = syn::parse2(TokenStream2::from(attr))
        .expect("A common trait must be specified: \"#[bundle(Trait)]\"");
    let item = TokenStream2::from(item);

    // parse enum body
    let mut e: ItemEnum = syn::parse2(item).expect("Bundle must be an enum.");

    assert!(
        !e.variants.is_empty(),
        "Bundle must contain at least one variant."
    );

    // transform into proper tuple variants
    e.variants = e
        .variants
        .iter()
        .map(|v| {
            let mut v = v.clone();

            match v.fields {
                Fields::Unit => {
                    let ident = v.ident.clone();

                    let mut punc = Punctuated::new();
                    punc.push(Field {
                        attrs: Vec::new(),
                        vis: Visibility::Inherited,
                        mutability: FieldMutability::None,
                        ident: None,
                        colon_token: None,
                        ty: Type::Verbatim(quote! { #ident }),
                    });

                    v.fields = Fields::Unnamed(FieldsUnnamed {
                        paren_token: Paren::default(),
                        unnamed: punc,
                    });

                    v
                }
                Fields::Unnamed(_) => v,
                Fields::Named(_) => panic!("Bundles cannot contain struct variants."),
            }
        })
        .collect();

    // extract visibility, ident, variant idents/types, and generics for generation
    let vis = e.vis.clone();
    let ident = e.ident.clone();
    let (impl_generics, ty_generics, where_clause) = e.generics.split_for_impl().clone();
    let variant_idents: Vec<Ident> = e.variants.iter().cloned().map(|v| v.ident).collect();
    let variant_tys: Vec<Type> = e
        .variants
        .iter()
        .cloned()
        .map(|v| match v.fields {
            Fields::Unnamed(fields) => fields.unnamed.first().unwrap().ty.clone(),
            _ => {
                unreachable!("All variant fields are unnamed by now.")
            }
        })
        .collect();

    // generate variant variables for match statement
    let variant_vars: Vec<Ident> = variant_idents
        .iter()
        .map(|i| {
            Ident::new(
                &inflector::cases::snakecase::to_snake_case(&i.to_string()),
                Span::call_site(),
            )
        })
        .collect();

    // validate variants hold eactly one type
    for variant in &e.variants {
        if variant.fields.len() != 1 {
            panic!("Bundle variants must hold exacly one type.")
        }
    }

    quote! {
        #e

        // From's for each variant of the bundle
        #(
            impl #impl_generics From<#variant_tys> for #ident #ty_generics #where_clause {
                #[inline]
                fn from(value: #variant_tys) -> Self {
                    Self::#variant_idents(value)
                }
            }
        )*

        // impl for inner func
        impl #impl_generics #ident #ty_generics #where_clause {
            #vis fn inner(&mut self) -> &mut dyn #tr {
                match self {
                    #(
                        #ident::#variant_idents(#variant_vars) => #variant_vars
                    ),*
                }
            }
        }
    }
    .into()
}
