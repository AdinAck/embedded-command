use core::{marker::PhantomData, mem::MaybeUninit};

use fill_array::fill;

use super::Encoding;

use crate::{Medium, SerializeBuf, SerializeIter, error};

// reexport proc macros
pub use macros::{SerializeBuf, SerializeIter};

pub struct Vanilla;

impl Encoding for Vanilla {
    type Word = u8;
    type Serialized<const SIZE: usize> = [Self::Word; SIZE];
}

impl<const N: usize> Medium for [u8; N] {
    fn default() -> Self {
        [0; N]
    }
}

macro_rules! impl_number {
    ($TYPE:ty, $SIZE:expr) => {
        impl SerializeIter for $TYPE {
            fn serialize_iter<'a>(
                &self,
                dst: impl IntoIterator<Item = &'a mut <Vanilla as Encoding>::Word>,
            ) -> Result<usize, error::EndOfInput>
            where
                <Vanilla as Encoding>::Word: 'a,
            {
                let mut dst = dst.into_iter();

                // 1. vanilla encoding uses bytes
                // 2. length constraint is on dest, not the type
                // 3. le_bytes because most no_std targets are LE native
                for byte in self.to_le_bytes() {
                    *dst.next().ok_or(error::EndOfInput)? = byte;
                }

                Ok($SIZE)
            }

            fn deserialize_iter<'a>(
                src: impl IntoIterator<Item = &'a <Vanilla as Encoding>::Word>,
            ) -> Result<Self, error::Error>
            where
                <Vanilla as Encoding>::Word: 'a,
            {
                let mut src = src.into_iter();

                // 1. vanilla encoding uses bytes
                // 2. all byte values are valid
                let bytes = fill![*src.next().ok_or(error::EndOfInput)?; $SIZE];

                // le_bytes because most no_std targets are LE native
                Ok(Self::from_le_bytes(bytes))
            }
        }

        // SAFETY: $SIZE must be correct as it is validated by it's usage with `from_le_bytes`
        unsafe impl SerializeBuf for $TYPE {
            const SIZE: usize = $SIZE;
        }
    };
}

// number impls

// isize/usize have platform specific size!
// NOTE: getting the "size" values wrong here
// will result in a compile-timer error, not UB
impl_number!(u8, 1);
impl_number!(u16, 2);
impl_number!(u32, 4);
impl_number!(u64, 8);
impl_number!(i8, 1);
impl_number!(i16, 2);
impl_number!(i32, 4);
impl_number!(i64, 8);
impl_number!(f32, 4);
impl_number!(f64, 8);

// bool impls

impl SerializeIter for bool {
    fn serialize_iter<'a>(
        &self,
        dst: impl IntoIterator<Item = &'a mut <Vanilla as Encoding>::Word>,
    ) -> Result<usize, error::EndOfInput>
    where
        <Vanilla as Encoding>::Word: 'a,
    {
        let mut dst = dst.into_iter();

        *dst.next().ok_or(error::EndOfInput)? = if *self { 1 } else { 0 };

        Ok(1)
    }

    fn deserialize_iter<'a>(
        src: impl IntoIterator<Item = &'a <Vanilla as Encoding>::Word>,
    ) -> Result<Self, error::Error>
    where
        <Vanilla as Encoding>::Word: 'a,
    {
        let mut src = src.into_iter();

        match *src.next().ok_or(error::EndOfInput)? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(error::Invalid)?,
        }
    }
}

unsafe impl SerializeBuf for bool {
    const SIZE: usize = 1;
}

// array impls

impl<T: SerializeIter, const N: usize> SerializeIter for [T; N] {
    fn serialize_iter<'a>(
        &self,
        dst: impl IntoIterator<Item = &'a mut <Vanilla as Encoding>::Word>,
    ) -> Result<usize, error::EndOfInput>
    where
        <Vanilla as Encoding>::Word: 'a,
    {
        let mut dst = dst.into_iter();
        let mut used = 0;

        for item in self {
            used += item.serialize_iter(&mut dst)?;
        }

        Ok(used)
    }

    fn deserialize_iter<'a>(
        src: impl IntoIterator<Item = &'a <Vanilla as Encoding>::Word>,
    ) -> Result<Self, error::Error>
    where
        <Vanilla as Encoding>::Word: 'a,
    {
        let mut src = src.into_iter();

        // `MaybeUninit` is used to avoid a `Default` requirement
        // SAFETY: `result` is purely written to
        let mut result: [MaybeUninit<T>; N] = unsafe { MaybeUninit::uninit().assume_init() };

        for value in result.iter_mut() {
            value.write(T::deserialize_iter(&mut src)?);
        }

        // SAFETY: by now all elements are initialized
        Ok(result.map(|e| unsafe { e.assume_init() }))
    }
}

// implementing `SerializeBuf` for generic arrays requires the "generic_const_exprs" feature

unsafe impl<T: SerializeBuf, const N: usize> SerializeBuf for [T; N] {
    const SIZE: usize = T::SIZE * N;
}

// tuple impls

macro_rules! impl_tuple {
    ( $(($TYPE:ident, $NAME:ident)),+ ) => {
        impl<$($TYPE: SerializeIter),+> SerializeIter for ($($TYPE,)+) {
            fn serialize_iter<'a>(
                &self,
                dst: impl IntoIterator<Item = &'a mut <Vanilla as Encoding>::Word>,
            ) -> Result<usize, error::EndOfInput>
            where
                <Vanilla as Encoding>::Word: 'a,
            {
                let mut dst = dst.into_iter();
                let mut used = 0;

                let ($($NAME,)+) = self;

                $(
                    used += $NAME.serialize_iter(&mut dst)?;
                )+

                Ok(used)
            }

            fn deserialize_iter<'a>(
                src: impl IntoIterator<Item = &'a <Vanilla as Encoding>::Word>,
            ) -> Result<Self, error::Error>
            where
                <Vanilla as Encoding>::Word: 'a,
            {
                let mut src = src.into_iter();

                $(
                    let $NAME = $TYPE::deserialize_iter(&mut src)?;
                )+

                Ok(($($NAME,)+))
            }
        }

        unsafe impl<$($TYPE: SerializeBuf),+> SerializeBuf for ($($TYPE,)+) {
            const SIZE: usize = $($TYPE::SIZE+)+0;
        }
    };
}

// implementing `SerializeBuf` for generic tuples requires the "generic_const_exprs" feature

// NOTE: incorrect macro arguments will result in compile-time error, not UB
impl_tuple!((A, a));
impl_tuple!((A, a), (B, b));
impl_tuple!((A, a), (B, b), (C, c));
impl_tuple!((A, a), (B, b), (C, c), (D, d));
impl_tuple!((A, a), (B, b), (C, c), (D, d), (E, e));
impl_tuple!((A, a), (B, b), (C, c), (D, d), (E, e), (F, f));
impl_tuple!((A, a), (B, b), (C, c), (D, d), (E, e), (F, f), (G, g));

// PhantomData impl (no-op)

impl<T> SerializeIter for PhantomData<T> {
    fn serialize_iter<'a>(
        &self,
        _dst: impl IntoIterator<Item = &'a mut <Vanilla as Encoding>::Word>,
    ) -> Result<usize, error::EndOfInput>
    where
        <Vanilla as Encoding>::Word: 'a,
    {
        Ok(0)
    }

    fn deserialize_iter<'a>(
        _src: impl IntoIterator<Item = &'a <Vanilla as Encoding>::Word>,
    ) -> Result<Self, error::Error>
    where
        <Vanilla as Encoding>::Word: 'a,
    {
        Ok(PhantomData)
    }
}

#[cfg(test)]
mod tests {
    mod primitives {
        use crate as serac;
        use serac::{SerializeIter, error};

        macro_rules! iter_test {
            ($TYPE:ty) => {
                let mut buf = [0; 8];

                // introduce some basic value differences
                let test_num = <$TYPE>::MAX / (0xa as $TYPE);

                test_num.serialize_iter(&mut buf).unwrap();
                let read_num = <$TYPE>::deserialize_iter(&buf).unwrap();

                assert_eq!(test_num, read_num);
            };
        }

        #[test]
        fn iter() {
            // numbers

            iter_test!(u8);
            iter_test!(u16);
            iter_test!(u32);
            iter_test!(u64);
            iter_test!(i8);
            iter_test!(i16);
            iter_test!(i32);
            iter_test!(i64);
            iter_test!(f32);
            iter_test!(f64);

            // bool

            let mut buf = [0; 1];

            // check valid values
            for val in [false, true] {
                val.serialize_iter(&mut buf).unwrap();

                assert_eq!(val, bool::deserialize_iter(&buf).unwrap());
            }

            // check invalid values
            for num in 2..=u8::MAX {
                num.serialize_iter(&mut buf).unwrap();

                match bool::deserialize_iter(&buf) {
                    Err(error::Error::Invalid) => {}
                    _ => panic!(),
                }
            }
        }
    }

    // rust analyzer cannot cope with recursive crate import
    #[cfg(test)]
    mod derive {
        use core::marker::PhantomData;

        use crate as serac; // for the proc macro
        use serac::{SerializeBuf, SerializeIter, buf, encoding::vanilla};

        mod structs {
            use super::*;

            #[derive(Debug, PartialEq, vanilla::SerializeIter, vanilla::SerializeBuf)]
            struct Foo {
                a: u8,
                b: i16,
            }

            #[derive(Debug, PartialEq, vanilla::SerializeIter, vanilla::SerializeBuf)]
            struct Nothing;

            #[derive(Debug, PartialEq, vanilla::SerializeIter, vanilla::SerializeBuf)]
            struct Bar(u8, Nothing, i16);

            #[derive(Debug, PartialEq, vanilla::SerializeIter, vanilla::SerializeBuf)]
            struct Baz {
                numbers: [f32; 16],
                flags: (bool, u8),
            }

            #[test]
            fn iter() {
                let mut buf = buf!(Foo);
                assert_eq!(3, buf.len());

                let test_foo = Foo { a: 0xaa, b: -1 };
                assert_eq!(3, test_foo.serialize_iter(&mut buf).unwrap());

                let read_foo = Foo::deserialize_iter(&buf).unwrap();

                assert_eq!(test_foo, read_foo);

                let mut buf = buf!(Bar);
                assert_eq!(3, buf.len());

                let test_bar = Bar(0xaa, Nothing, -1);
                assert_eq!(3, test_bar.serialize_iter(&mut buf).unwrap());

                let read_bar = Bar::deserialize_iter(&buf).unwrap();

                assert_eq!(test_bar, read_bar);

                let mut buf = buf!(Baz);
                assert_eq!(66, buf.len());

                let test_baz = Baz {
                    numbers: [0.; 16],
                    flags: (false, 0xaa),
                };
                assert_eq!(66, test_baz.serialize_iter(&mut buf).unwrap());

                let read_baz = Baz::deserialize_iter(&buf).unwrap();

                assert_eq!(test_baz, read_baz);
            }

            #[test]
            fn buf() {
                let mut buf = buf!(Foo);
                assert_eq!(3, buf.len());

                let test_foo = Foo { a: 0xaa, b: -1 };
                assert_eq!(3, test_foo.serialize_buf(&mut buf));

                let read_foo = Foo::deserialize_buf(&buf).unwrap();

                assert_eq!(test_foo, read_foo);

                let mut buf = buf!(Bar);
                assert_eq!(3, buf.len());

                let test_bar = Bar(0xaa, Nothing, -1);
                assert_eq!(3, test_bar.serialize_buf(&mut buf));

                let read_bar = Bar::deserialize_buf(&buf).unwrap();

                assert_eq!(test_bar, read_bar);

                let mut buf = buf!(Baz);
                assert_eq!(66, buf.len());

                let test_baz = Baz {
                    numbers: [0.; 16],
                    flags: (false, 0xaa),
                };
                assert_eq!(66, test_baz.serialize_buf(&mut buf));

                let read_baz = Baz::deserialize_buf(&buf).unwrap();

                assert_eq!(test_baz, read_baz);
            }
        }

        mod enums {
            use super::*;

            const BE: u8 = 0xbe;

            #[derive(Debug, PartialEq, vanilla::SerializeIter, vanilla::SerializeBuf)]
            #[repr(u8)]
            enum Foo {
                A,
                B(u8, i16) = 0xde,
                C,
                D { bar: u16, t: i8 } = BE,
            }

            #[test]
            fn iter() {
                let mut buf = buf!(Foo);
                assert_eq!(4, buf.len());

                let test_foo = Foo::D { bar: 0xaa, t: -1 };
                assert_eq!(4, test_foo.serialize_iter(&mut buf).unwrap());

                let read_foo = Foo::deserialize_iter(&buf).unwrap();

                assert_eq!(test_foo, read_foo);

                let test_foo = Foo::A;
                assert_eq!(1, test_foo.serialize_iter(&mut buf).unwrap());

                let read_foo = Foo::deserialize_iter(&buf).unwrap();

                assert_eq!(test_foo, read_foo);
            }

            #[test]
            fn buf() {
                let mut buf = buf!(Foo);
                assert_eq!(4, buf.len());

                let test_foo = Foo::D { bar: 0xaa, t: -1 };
                assert_eq!(4, test_foo.serialize_buf(&mut buf));

                let read_foo = Foo::deserialize_buf(&buf).unwrap();

                assert_eq!(test_foo, read_foo);

                let test_foo = Foo::A;
                assert_eq!(1, test_foo.serialize_buf(&mut buf));

                let read_foo = Foo::deserialize_buf(&buf).unwrap();

                assert_eq!(test_foo, read_foo);
            }
        }

        #[test]
        fn generics() {
            const BE: u8 = 0xbe;

            #[derive(Debug, PartialEq, vanilla::SerializeIter)]
            #[repr(u16)]
            enum FooGen<T, U>
            where
                T: SerializeIter,
                U: SerializeIter,
            {
                A(u8, T),
                B { woah: U } = BE as u16, // arbitrary expression in discriminant!
            }

            #[derive(Debug, PartialEq, vanilla::SerializeIter)]
            struct BarGen<T>
            where
                T: SerializeIter + SerializeBuf,
            {
                a: T,
                b: FooGen<bool, T>,
                c: PhantomData<T>,
            }

            let mut buf = [0; 4];

            let test_bar = BarGen {
                a: -1i16,
                b: FooGen::A(0xaa, false),
                c: PhantomData,
            };

            // buf is too small
            assert!(test_bar.serialize_iter(&mut buf).is_err());

            let mut buf = [0; 8];

            assert_eq!(6, test_bar.serialize_iter(&mut buf).unwrap());

            let read_bar = SerializeIter::deserialize_iter(&buf).unwrap();

            assert_eq!(test_bar, read_bar); // comparison provides type inference for deserialization!
        }
    }
}
