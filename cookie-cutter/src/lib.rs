//! A static, modular, and light-weight serialization framework.

#![no_std]

pub mod encoding;
pub mod medium;

use core::hint::unreachable_unchecked;

use encoding::{vanilla::Vanilla, Encoding};
use medium::Medium;

pub mod error {
    #[derive(Debug, Clone, Copy)]
    #[cfg_attr(feature = "defmt", derive(defmt::Format))]
    pub struct EndOfInput;

    #[derive(Debug, Clone, Copy)]
    #[cfg_attr(feature = "defmt", derive(defmt::Format))]
    pub struct Invalid;

    #[derive(Debug, Clone, Copy)]
    #[cfg_attr(feature = "defmt", derive(defmt::Format))]
    pub enum Error {
        EndOfInput,
        Invalid,
    }

    impl From<EndOfInput> for Error {
        fn from(_: EndOfInput) -> Self {
            Self::EndOfInput
        }
    }

    impl From<Invalid> for Error {
        fn from(_: Invalid) -> Self {
            Self::Invalid
        }
    }
}

/// This trait defines a highly adaptable interface
/// for serializing and deserializing types to and
/// from a serialization medium via iterators.
pub trait SerializeIter<E: Encoding = Vanilla>: Sized {
    /// Serialize the implementer type to a
    /// serialization medium via an iterator.
    fn serialize_iter<'a>(
        &self,
        dst: impl IntoIterator<Item = &'a mut E::Word>,
    ) -> Result<(), error::EndOfInput>
    where
        E::Word: 'a;

    /// Deserialize the implementer type from a
    /// serialization medium via an iterator.
    fn deserialize_iter<'a>(
        src: impl IntoIterator<Item = &'a E::Word>,
    ) -> Result<Self, error::Error>
    where
        E::Word: 'a;
}

/// This trait defines a more rigid/static serialization
/// interface.
///
/// Types that implement this trait can be serialized to
/// and from buffers with an exact length. This length
/// being the maximum needed for any value of the
/// implementer type.
///
/// To implement this trait, the type must already implement
/// `SerializeIter` and the implementer must compute
/// the necessary length of the serialization medium.
///
/// # Safety
///
/// The length of the associated `Serialized` type is critical.
/// An insufficient length *will* result in UB. Best to leave
/// this implementation to the procedural macro.
pub unsafe trait SerializeBuf<E: Encoding = Vanilla>: SerializeIter<E> {
    /// The type respresenting the serialized form of the implementer type.
    type Serialized: Medium<E>;

    /// Serialize the implementer type to a
    /// serialization medium.
    fn serialize_buf(&self, dest: &mut Self::Serialized) {
        // SAFETY: dependent on safety of trait implementation.
        // `Serialized` must be of sufficient length.
        unsafe { SerializeIter::serialize_iter(self, dest.get_iter_mut()).unwrap_unchecked() };
    }

    /// Deserialize the implementer type from a
    /// serialization medium.
    fn deserialize_buf(src: &Self::Serialized) -> Result<Self, error::Invalid> {
        SerializeIter::deserialize_iter(src.get_iter()).or_else(|err| match err {
            error::Error::Invalid => Err(error::Invalid),
            // SAFETY: dependent on safety of trait implementation.
            // `Serialized` must be of sufficient length.
            error::Error::EndOfInput => unsafe { unreachable_unchecked() },
        })
    }
}
