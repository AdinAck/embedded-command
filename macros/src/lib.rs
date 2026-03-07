use proc_macro::TokenStream;

mod dispatch_bundle;
mod serac;

/// Transform attached enum into a "bundle".
///
/// *What is a bundle?*
///
/// Bundles are used to accomplish dynamic dispatch in resource constrained systems (no_std).
/// A bundle can hold a finite number of types that implement a common trait.
/// The size of the bundle is known at compile time and equal to the size of the largest type in the bundle.
/// Bundles are useful for type-erasure when transporting multiple types pseudo-heterogeneously.
#[proc_macro_attribute]
pub fn bundle(attr: TokenStream, item: TokenStream) -> TokenStream {
    dispatch_bundle::bundle(attr, item)
}

/// Generates the implementation block for conforming to `SerializeIter` of the "vanilla" flavor.
///
/// # Note
///
/// Requires `serac` to be in scope with that name.
#[proc_macro_derive(SerializeIter)]
pub fn impl_serialize_iter_vanilla(item: TokenStream) -> TokenStream {
    serac::vanilla::serialize_iter(item)
}

/// Generates the implementation block for conforming to `Size` of the "vanilla" flavor.
///
/// # Note
///
/// Requires `serac` to be in scope with that name.
#[proc_macro_derive(Size)]
pub fn impl_size_vanilla(item: TokenStream) -> TokenStream {
    serac::vanilla::impl_size(item)
}

/// Generates the implementation block for conforming to `SerializeBuf`.
///
/// As of now, generic types *cannot* implement `SerializeBuf` on stable.
///
/// # Note
///
/// Requires `serac` to be in scope with that name.
#[proc_macro_derive(SerializeBuf)]
pub fn impl_serialize_buf(item: TokenStream) -> TokenStream {
    serac::impl_serialize_buf(item)
}

/// Generates the implementation block for conforming to `SerializeBuf` for a type
/// alias. This is used for implementing `SerializeBuf` for concretely specified
/// generic types.
///
/// # Note
///
/// Requires `serac` to be in scope with that name.
#[proc_macro_attribute]
pub fn impl_serialize_buf_alias(attrs: TokenStream, item: TokenStream) -> TokenStream {
    serac::impl_serialize_buf_alias(attrs, item)
}
