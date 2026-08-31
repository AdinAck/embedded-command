//! Vanilla encoding implementations for types from the [`heapless`] crate.

use heapless::Vec;

use crate::{SerializeIter, Size, error};

pub type VecU8<T, const N: usize> = Vec<T, N, u8>;
pub type VecU16<T, const N: usize> = Vec<T, N, u16>;
pub type VecU32<T, const N: usize> = Vec<T, N, u32>;
pub type VecU64<T, const N: usize> = Vec<T, N, u64>;

impl<T: SerializeIter, LenT: heapless::LenType + SerializeIter, const N: usize> SerializeIter
    for Vec<T, N, LenT>
{
    fn ser<'a>(
        &self,
        dst: &mut crate::Buf<
            impl Iterator<Item = &'a mut <super::Vanilla as crate::Encoding>::Word>,
        >,
    ) -> Result<(), crate::error::EndOfInput>
    where
        <super::Vanilla as crate::Encoding>::Word: 'a,
    {
        LenT::from_usize(self.len()).ser(dst)?;

        for e in self {
            e.ser(dst)?;
        }

        Ok(())
    }

    fn de<'a>(
        src: &mut crate::Buf<impl Iterator<Item = &'a <super::Vanilla as crate::Encoding>::Word>>,
    ) -> Result<Self, crate::error::Error>
    where
        <super::Vanilla as crate::Encoding>::Word: 'a,
    {
        let mut vec = Vec::new();

        let len = LenT::de(src)?;

        // invariant 0: encoded length must not exceed vec capacity
        if len.into_usize() > N {
            Err(error::Invalid)?
        }

        for _ in 0..len.into_usize() {
            // SAFETY: room is ensured by invariant 0
            unsafe { vec.push_unchecked(T::de(src)?) };
        }

        Ok(vec)
    }
}

// SAFETY: size of len + size of element * number of elements
unsafe impl<T: Size, LenT: heapless::LenType + Size, const N: usize> Size for Vec<T, N, LenT> {
    const SIZE: usize = LenT::SIZE + T::SIZE * N;
}

#[cfg(test)]
mod tests {
    mod vec {

        use crate::{
            self as serac, SerializeIter, Size, buf,
            encoding::vanilla::heapless::{VecU8, VecU32},
        };

        #[test]
        fn simple() {
            let v = const { VecU8::<u8, 255>::from_array([5, 4, 3, 2, 1, 0]) };

            let mut buf = buf!(VecU8<u8, 255>);

            v.serialize_iter(&mut buf).expect("vec should fit in buf");

            let readback = VecU8::<u8, 255>::deserialize_iter(&buf)
                .expect("vec should deserialize successfully");

            assert_eq!(7, readback.used());
            itertools::assert_equal(v, readback.take());
        }

        #[test]
        fn shorter() {
            let v = const { VecU8::<u8, 255>::from_array([5, 4, 3, 2, 1, 0]) };

            let mut buf = buf!(VecU8<u8, 255>);

            v.serialize_iter(&mut buf).expect("vec should fit in buf");

            let readback = VecU8::<u8, 6>::deserialize_iter(&buf)
                .expect("vec should deserialize successfully");

            assert_eq!(7, readback.used());
            itertools::assert_equal(v, readback.take());
        }

        #[test]
        fn too_short() {
            let v = const { VecU8::<u8, 255>::from_array([5, 4, 3, 2, 1, 0]) };

            let mut buf = buf!(VecU8<u8, 255>);

            v.serialize_iter(&mut buf).expect("vec should fit in buf");

            let readback = VecU8::<u8, 5>::deserialize_iter(&buf);

            assert!(
                readback.is_err(),
                "expected deserialization to fail since the vec has an insufficient capacity",
            );
        }

        #[test]
        fn buf_too_small() {
            let v = const { VecU8::<u8, 255>::from_array([5, 4, 3, 2, 1, 0]) };

            let mut buf = buf!(VecU8<u8, 5>);

            assert!(
                v.serialize_iter(&mut buf).is_err(),
                "expected serialization to fail since the buffer has an insufficient capacity",
            );
        }

        #[test]
        fn len_type() {
            assert_eq!(
                VecU32::<(), 0>::SIZE,
                <u32 as Size>::SIZE,
                "expected empty vec with u32 length type to be the same size as u32",
            );
        }
    }
}
