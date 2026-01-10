pub mod vanilla;
pub use vanilla::Vanilla;

/// Types implement this trait to be used as indication of a specific encoding
/// scheme.
pub trait Encoding {
    /// The fundamental word of the encoding scheme.
    ///
    /// i.e. `u8` for `[u8; ...]` mediums.
    type Word;

    /// The serialized form targeted by this encoding scheme.
    type Serialized<const SIZE: usize>;
}
