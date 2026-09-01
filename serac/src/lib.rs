//! A static, modular, and light-weight serialization framework.

#![no_std]

mod buf;
pub mod encoding;
pub mod medium;
mod transport;

pub use buf::Buf;
use derive_more::{Deref, DerefMut};
pub use encoding::Encoding;
use encoding::vanilla::Vanilla;
pub use error::Error;
pub use macros::{SerializeBuf, impl_serialize_buf_alias as serialize_buf};
pub use medium::Medium;
use ters::ters;
pub use transport::Transport;

pub mod error {
    use crate as serac;
    use serac::encoding::vanilla;

    /// The encoder reached the end of the input before serialization/deserialization
    /// was complete.
    #[derive(Debug, Clone, Copy)]
    #[cfg_attr(feature = "defmt", derive(defmt::Format))]
    pub struct EndOfInput;

    /// The contents of the serialization medium produced an invalid deserialized
    /// value.
    #[derive(Debug, Clone, Copy)]
    #[cfg_attr(feature = "defmt", derive(defmt::Format))]
    pub struct Invalid;

    /// Deserialization failed.
    #[repr(u8)]
    #[derive(Debug, Clone, Copy, vanilla::SerializeIter, vanilla::Size, serac::SerializeBuf)]
    #[cfg_attr(feature = "defmt", derive(defmt::Format))]
    pub enum Error {
        /// The encoder reached the end of the input before deserialization was
        /// complete.
        EndOfInput,
        /// The contents of the serialization medium produced an invalid deserialized
        /// value.
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

/// This trait defines a highly adaptable interface for serializing and deserializing
/// types to and from a serialization medium via iterators.
pub trait SerializeIter<E: Encoding = Vanilla>: Sized {
    /// Serialize the implementer type to a serialization medium via an iterator.
    fn serialize_iter<'a>(
        &self,
        dst: impl IntoIterator<Item = &'a mut E::Word>,
    ) -> Result<usize, error::EndOfInput>
    where
        E::Word: 'a,
    {
        let mut buf = buf::Buf::from(dst);
        self.ser(&mut buf)?;
        Ok(buf.used)
    }

    /// Deserialize the implementer type from a serialization medium via an iterator.
    fn deserialize_iter<'a>(
        src: impl IntoIterator<Item = &'a E::Word>,
    ) -> Result<Deserialized<Self>, error::Error>
    where
        E::Word: 'a,
    {
        let mut buf = Buf::from(src);
        // for now, the number of bytes used is discarded
        Ok(Deserialized {
            value: Self::de(&mut buf)?,
            used: buf.used,
        })
    }

    /// Inner serialization method, wrapped by [`serialize_iter`](SerializeIter::serialize_iter).
    fn ser<'a>(
        &self,
        dst: &mut Buf<impl Iterator<Item = &'a mut E::Word>>,
    ) -> Result<(), error::EndOfInput>
    where
        E::Word: 'a;

    /// Inner deserialization function, wrapped by [`deserialize_iter`](SerializeIter::deserialize_iter).
    fn de<'a>(src: &mut Buf<impl Iterator<Item = &'a E::Word>>) -> Result<Self, error::Error>
    where
        E::Word: 'a;
}

/// This trait defines a more rigid/static serialization interface.
///
/// Types that implement this trait can be serialized to and from buffers with an
/// exact length. This length being the minimum needed for any value of the
/// implementer type.
///
/// To implement this trait, the type must already implement [`SerializeIter`] and [`Size`].
///
/// # Safety
///
/// This trait must only be implemented for all `T` where `T: Size` and
/// `N == T::SIZE`.
pub unsafe trait SerializeBuf<const N: usize, E: Encoding = Vanilla>:
    SerializeIter<E> + Size<E>
{
    fn serialize_buf<'a>(&self, buf: &'a mut E::Serialized<N>) -> usize
    where
        &'a mut E::Serialized<N>: IntoIterator<Item = &'a mut E::Word>,
        E::Word: 'a,
    {
        unsafe { SerializeIter::serialize_iter(self, buf).unwrap_unchecked() }
    }

    fn deserialize_buf<'a>(src: &'a E::Serialized<N>) -> Result<Deserialized<Self>, error::Invalid>
    where
        &'a E::Serialized<N>: IntoIterator<Item = &'a E::Word>,
        E::Word: 'a,
    {
        SerializeIter::deserialize_iter(src).or_else(|err| match err {
            error::Error::Invalid => Err(error::Invalid),
            // SAFETY: dependent on safety of trait implementation.
            // `Serialized` must be of sufficient length.
            error::Error::EndOfInput => unsafe { ::core::hint::unreachable_unchecked() },
        })
    }
}

/// This trait allows implementors to define the serialized size according to the encoding scheme.
///
/// # Safety
///
/// The value of the associated `SIZE` constant is critical. An insufficient size
/// *will* result in UB. Best to leave this implementation to the procedural macro.
pub unsafe trait Size<E = Vanilla> {
    /// The size of the implementor when serialized, according to the encoding
    /// scheme.
    const SIZE: usize;
}

/// A successfully deserialized value. Use [`Deref`](::core::ops::Deref), [`DerefMut`](::core::ops::DerefMut), or
/// [`take`](Deserialized::take) to access the inner `T`. Use [`used`](Deserialized::used) to view the number of words
/// used to deserialize the `T`.
#[ters]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deref, DerefMut)]
#[cfg_attr(feature = "defmt", derive(::defmt::Format))]
pub struct Deserialized<T> {
    #[deref]
    #[deref_mut]
    value: T,
    #[get(deref)]
    used: usize,
}

impl<T> Deserialized<T> {
    /// Take the deserialized value from the container.
    pub fn take(self) -> T {
        self.value
    }
}

/// Create an empty buffer for the provided type serialized with the provided
/// encoding scheme. Optionally, a multiplier may be provided which is multiplied by
/// the minimum buffer size.
#[macro_export]
macro_rules! buf {
    ($ty:ty: $enc:ty $(, $coef:expr)?) => {
        <<$enc as serac::Encoding>::Serialized<{ <$ty as serac::Size>::SIZE $(*$coef)? }> as serac::Medium>::default()
    };
    ($ty:ty $(, $coef:expr)?) => {
        buf!($ty: serac::encoding::Vanilla $(, $coef)?)
    };
}
