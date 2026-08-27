use crate::{Buf, Encoding, SerializeIter, Size, encoding::Vanilla, error};

impl<T: SerializeIter> SerializeIter for Option<T> {
    fn ser<'a>(
        &self,
        dst: &mut Buf<impl Iterator<Item = &'a mut <Vanilla as Encoding>::Word>>,
    ) -> Result<(), error::EndOfInput>
    where
        <Vanilla as Encoding>::Word: 'a,
    {
        match self {
            Some(t) => {
                true.ser(dst)?;
                t.ser(dst)
            }
            None => false.ser(dst),
        }
    }

    fn de<'a>(
        src: &mut Buf<impl Iterator<Item = &'a <Vanilla as Encoding>::Word>>,
    ) -> Result<Self, error::Error>
    where
        <Vanilla as Encoding>::Word: 'a,
    {
        Ok(match bool::de(src)? {
            true => Some(T::de(src)?),
            false => None,
        })
    }
}

// SAFETY: size of discriminant + size of T
unsafe impl<T: Size> Size for Option<T> {
    const SIZE: usize = 1 + T::SIZE;
}

#[cfg(test)]
mod tests {
    use crate::{self as serac, SerializeBuf as _, Size as _, buf};

    #[test]
    fn some() {
        #[serac::serialize_buf]
        type Test = Option<u32>;

        let mut buf = buf!(Test);

        let used = Some(0xdeadbeef).serialize_buf(&mut buf);

        assert_eq!(used, u32::SIZE + 1);
    }

    #[test]
    fn none() {
        let mut buf = [0; _];

        // note: inferred `Option<u32>` from `SerializeBuf` impl in `some`
        let used = None.serialize_buf(&mut buf);

        assert_eq!(used, 1);
    }
}
