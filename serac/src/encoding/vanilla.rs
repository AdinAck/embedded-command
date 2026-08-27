mod core;
#[cfg(feature = "heapless")]
pub mod heapless;

use ::core::mem::MaybeUninit;

use fill_array::fill;

use super::Encoding;

use crate::{Buf, Medium, SerializeBuf, SerializeIter, Size, error};

// reexport proc macros
pub use macros::{SerializeIter, Size};

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
            fn ser<'a>(
                &self,
                dst: &mut Buf<impl Iterator<Item = &'a mut <Vanilla as Encoding>::Word>>,
            ) -> Result<(), error::EndOfInput>
            where
                <Vanilla as Encoding>::Word: 'a,
            {
                // 1. vanilla encoding uses bytes
                // 2. length constraint is on dest, not the type
                // 3. le_bytes because most no_std targets are LE native
                for byte in self.to_le_bytes() {
                    *dst.next().ok_or(error::EndOfInput)? = byte;
                }

                Ok(())
            }

            fn de<'a>(
                src: &mut Buf<impl Iterator<Item = &'a <Vanilla as Encoding>::Word>>,
            ) -> Result<Self, error::Error>
            where
                <Vanilla as Encoding>::Word: 'a,
            {
                // 1. vanilla encoding uses bytes
                // 2. all byte values are valid
                let bytes = fill![*src.next().ok_or(error::EndOfInput)?; $SIZE];

                // le_bytes because most no_std targets are LE native
                Ok(Self::from_le_bytes(bytes))
            }
        }

        // SAFETY: $SIZE must be correct as it is validated by it's usage with `from_le_bytes`
        unsafe impl Size for $TYPE {
            const SIZE: usize = $SIZE;
        }

        unsafe impl SerializeBuf<{ <$TYPE as Size>::SIZE }> for $TYPE {}
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
    fn ser<'a>(
        &self,
        dst: &mut Buf<impl Iterator<Item = &'a mut <Vanilla as Encoding>::Word>>,
    ) -> Result<(), error::EndOfInput>
    where
        <Vanilla as Encoding>::Word: 'a,
    {
        *dst.next().ok_or(error::EndOfInput)? = if *self { 1 } else { 0 };

        Ok(())
    }

    fn de<'a>(
        src: &mut Buf<impl Iterator<Item = &'a <Vanilla as Encoding>::Word>>,
    ) -> Result<Self, error::Error>
    where
        <Vanilla as Encoding>::Word: 'a,
    {
        match *src.next().ok_or(error::EndOfInput)? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(error::Invalid)?,
        }
    }
}

unsafe impl Size for bool {
    const SIZE: usize = 1;
}

// array impls

impl<T: SerializeIter, const N: usize> SerializeIter for [T; N] {
    fn ser<'a>(
        &self,
        dst: &mut Buf<impl Iterator<Item = &'a mut <Vanilla as Encoding>::Word>>,
    ) -> Result<(), error::EndOfInput>
    where
        <Vanilla as Encoding>::Word: 'a,
    {
        for item in self {
            item.ser(dst)?;
        }

        Ok(())
    }

    fn de<'a>(
        src: &mut Buf<impl Iterator<Item = &'a <Vanilla as Encoding>::Word>>,
    ) -> Result<Self, error::Error>
    where
        <Vanilla as Encoding>::Word: 'a,
    {
        // `MaybeUninit` is used to avoid a `Default` requirement
        // SAFETY: `result` is purely written to
        let mut result: [MaybeUninit<T>; N] = unsafe { MaybeUninit::uninit().assume_init() };

        for value in result.iter_mut() {
            value.write(T::de(src)?);
        }

        // SAFETY: by now all elements are initialized
        Ok(result.map(|e| unsafe { e.assume_init() }))
    }
}

// implementing `SerializeBuf` for generic arrays requires the "generic_const_exprs" feature

unsafe impl<T: Size, const N: usize> Size for [T; N] {
    const SIZE: usize = T::SIZE * N;
}

// tuple impls

macro_rules! impl_tuple {
    ( $(($TYPE:ident, $NAME:ident)),+ ) => {
        impl<$($TYPE: SerializeIter),+> SerializeIter for ($($TYPE,)+) {
            fn ser<'a>(
                &self,
                dst: &mut Buf<impl Iterator<Item = &'a mut <Vanilla as Encoding>::Word>>,
            ) -> Result<(), error::EndOfInput>
            where
                <Vanilla as Encoding>::Word: 'a,
            {
                let ($($NAME,)+) = self;

                $(
                    $NAME.ser(dst)?;
                )+

                Ok(())
            }

            fn de<'a>(
                src: &mut Buf<impl Iterator<Item = &'a <Vanilla as Encoding>::Word>>,
            ) -> Result<Self, error::Error>
            where
                <Vanilla as Encoding>::Word: 'a,
            {
                $(
                    let $NAME = $TYPE::de(src)?;
                )+

                Ok(($($NAME,)+))
            }
        }

        unsafe impl<$($TYPE: Size),+> Size for ($($TYPE,)+) {
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

// unit impl (no-op)

impl SerializeIter for () {
    fn ser<'a>(
        &self,
        _dst: &mut Buf<impl Iterator<Item = &'a mut <Vanilla as Encoding>::Word>>,
    ) -> Result<(), error::EndOfInput>
    where
        <Vanilla as Encoding>::Word: 'a,
    {
        Ok(())
    }

    fn de<'a>(
        _src: &mut Buf<impl Iterator<Item = &'a <Vanilla as Encoding>::Word>>,
    ) -> Result<Self, error::Error>
    where
        <Vanilla as Encoding>::Word: 'a,
    {
        Ok(())
    }
}

unsafe impl Size for () {
    const SIZE: usize = 0;
}

unsafe impl SerializeBuf<0> for () {}

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

            #[derive(
                Debug, PartialEq, vanilla::SerializeIter, vanilla::Size, serac::SerializeBuf,
            )]
            struct Foo {
                a: u8,
                b: i16,
            }

            #[derive(
                Debug, PartialEq, vanilla::SerializeIter, vanilla::Size, serac::SerializeBuf,
            )]
            struct Nothing;

            #[derive(
                Debug, PartialEq, vanilla::SerializeIter, vanilla::Size, serac::SerializeBuf,
            )]
            struct Bar(u8, Nothing, i16);

            #[derive(
                Debug, PartialEq, vanilla::SerializeIter, vanilla::Size, serac::SerializeBuf,
            )]
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

            #[derive(
                Debug, PartialEq, vanilla::SerializeIter, vanilla::Size, serac::SerializeBuf,
            )]
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

            #[derive(Debug, PartialEq, vanilla::SerializeIter, vanilla::Size)]
            #[repr(u16)]
            enum FooGen<T, U> {
                A(u8, T),
                B { woah: U } = BE as u16, // arbitrary expression in discriminant!
            }

            #[derive(Debug, PartialEq, vanilla::SerializeIter, vanilla::Size)]
            struct BarGen<T> {
                a: T,
                b: FooGen<bool, T>,
                c: PhantomData<T>,
            }

            #[serac::serialize_buf]
            type ConcreteFoo = FooGen<bool, i16>;

            let mut buf = [0; 4];

            let test_bar = BarGen {
                a: -1i16,
                b: FooGen::A(0xaa, false),
                c: PhantomData,
            };

            // buf is too small
            assert!(test_bar.serialize_iter(&mut buf).is_err());

            let mut buf = buf!(BarGen<i16>);

            assert_eq!(6, test_bar.serialize_iter(&mut buf).unwrap());

            let read_bar = SerializeIter::deserialize_iter(&buf).unwrap();

            assert_eq!(test_bar, read_bar); // comparison provides type inference for deserialization!

            let mut buf = buf!(ConcreteFoo);

            let test_foo = ConcreteFoo::B { woah: -42 };

            assert_eq!(4, test_foo.serialize_buf(&mut buf));

            let read_foo = SerializeBuf::deserialize_buf(&buf).unwrap();

            assert_eq!(test_foo, read_foo); // comparison provides type inference for deserialization!
        }
    }
}
