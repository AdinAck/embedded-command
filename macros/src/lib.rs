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

/// Generates the implementation block for conforming to `SerializeBuf` of the "vanilla" flavor.
///
/// As of now, generic types *cannot* implement `SerializeBuf` on stable.
///
/// # Note
///
/// Requires `serac` to be in scope with that name.
#[proc_macro_derive(SerializeBuf)]
pub fn impl_serialize_buf_vanilla(item: TokenStream) -> TokenStream {
    serac::vanilla::impl_serialize_buf(item)
}
