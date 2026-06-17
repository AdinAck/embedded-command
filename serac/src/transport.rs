use crate::{
    Buf, Encoding, SerializeBuf, SerializeIter, Size,
    encoding::Vanilla,
    error::{EndOfInput, Error},
};

/// Establishes a *transport* representation of the implementor for transparent conversion during serialization.
///
/// Types that implement this trait inherit the serialization capabilites and properties of the specified transport
/// type.
///
/// If `T: Transport<Transport = U>`, then:
///
/// - `T: SerializeIter<E>` where `U: SerializeIter<E>`
/// - `T: Size<E, SIZE = N>` where `U: Size<E, SIZE = N>`
/// - `T: SerializeBuf<E, N>` where `U: SerializeBuf<E, N>`
///
/// This is useful when the shape of a type is suboptimal for transport, but ergonomic in code.
///
/// For example, `[bool; 13]: Size<E = Vanilla, SIZE = 13>` (in other words, occupies 13 bytes with the vanilla
/// encoding). Rust presents no means for types to introspect, but a person could see this and know that the same
/// information only warrants 2 bytes over-the-wire. Implementing `Transport<Transport = u16>` for `[bool; 13]` would
/// cause `SerializeIter`, `Size`, and `SerializeBuf` to be implemented for `[bool; 13]` but with a size of `2` instead
/// of `13`.
pub trait Transport<E = Vanilla>: Copy {
    /// The transport type, which is the representation of the implementor type that is used in transport and converted
    /// to/from.
    type Transport;

    /// Convert the in-memory representation into the transport representation.
    fn into_transport(self) -> Self::Transport;

    /// Convert the transport representation into the in-memory representation.
    fn from_transport(transport: Self::Transport) -> Self;
}

impl<T, E> SerializeIter<E> for T
where
    T: Transport<E>,
    T::Transport: SerializeIter<E>,
    E: Encoding,
{
    fn ser<'a>(
        &self,
        dst: &mut Buf<impl Iterator<Item = &'a mut E::Word>>,
    ) -> Result<(), EndOfInput>
    where
        E::Word: 'a,
    {
        self.into_transport().ser(dst)
    }

    fn de<'a>(src: &mut Buf<impl Iterator<Item = &'a E::Word>>) -> Result<Self, Error>
    where
        E::Word: 'a,
    {
        Ok(Self::from_transport(T::Transport::de(src)?))
    }
}

unsafe impl<T, E> Size<E> for T
where
    T: Transport<E>,
    T::Transport: Size<E>,
    E: Encoding,
{
    const SIZE: usize = T::Transport::SIZE;
}

unsafe impl<T, E, const N: usize> SerializeBuf<N, E> for T
where
    T: Transport<E>,
    T::Transport: SerializeBuf<N, E>,
    E: Encoding,
{
}

#[cfg(test)]
mod tests {
    use crate as serac;

    use core::array;

    use serac::{SerializeBuf as _, Size as _, Transport, buf};

    #[test]
    fn transport() {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        struct Foo {
            flags: [bool; 5],
        }

        impl Transport for Foo {
            type Transport = u8;

            fn into_transport(self) -> Self::Transport {
                let mut word = 0;

                self.flags
                    .iter()
                    .enumerate()
                    .for_each(|(i, &flag)| word |= (flag as u8) << i);

                word
            }

            fn from_transport(transport: Self::Transport) -> Self {
                Self {
                    flags: array::from_fn(|i| match (transport >> i) & 1 {
                        0 => false,
                        1 => true,
                        _ => unreachable!(),
                    }),
                }
            }
        }

        let mut buf = buf!(Foo);

        assert_eq!(buf.len(), u8::SIZE);

        let foo = Foo {
            flags: [false, true, true, false, true],
        };

        foo.serialize_buf(&mut buf);

        let readback = Foo::deserialize_buf(&buf).unwrap();

        assert_eq!(foo, readback);
    }
}
