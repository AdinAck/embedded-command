pub mod vanilla;

/// Types implement this trait
/// to be used as indication of
/// a specific encoding scheme.
pub trait Encoding {
    /// The fundamental word of the
    /// encoding scheme.
    ///
    /// i.e. `u8` for `[u8; ...]` mediums.
    type Word;
}
