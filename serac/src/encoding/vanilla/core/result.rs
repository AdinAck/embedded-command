use crate::{Buf, Encoding, SerializeIter, Size, encoding::Vanilla, error};

impl<T: SerializeIter, E: SerializeIter> SerializeIter for Result<T, E> {
    fn ser<'a>(
        &self,
        dst: &mut Buf<impl Iterator<Item = &'a mut <Vanilla as Encoding>::Word>>,
    ) -> Result<(), error::EndOfInput>
    where
        <Vanilla as Encoding>::Word: 'a,
    {
        match self {
            Ok(t) => {
                true.ser(dst)?;
                t.ser(dst)
            }
            Err(e) => {
                false.ser(dst)?;
                e.ser(dst)
            }
        }
    }

    fn de<'a>(
        src: &mut Buf<impl Iterator<Item = &'a <Vanilla as Encoding>::Word>>,
    ) -> Result<Self, error::Error>
    where
        <Vanilla as Encoding>::Word: 'a,
    {
        Ok(match bool::de(src)? {
            true => Ok(T::de(src)?),
            false => Err(E::de(src)?),
        })
    }
}

// SAFETY: size of discriminant + max of size of T and size of E
unsafe impl<T: Size, E: Size> Size for Result<T, E> {
    const SIZE: usize = 1 + if T::SIZE > E::SIZE { T::SIZE } else { E::SIZE };
}

#[cfg(test)]
mod tests {
    use crate::{self as serac, SerializeBuf as _, Size as _, buf};

    #[test]
    fn ok() {
        #[serac::serialize_buf]
        type Test = Result<u32, ()>;

        let mut buf = buf!(Test);

        let used = Ok(0xdeadbeef).serialize_buf(&mut buf);

        assert_eq!(used, u32::SIZE + 1);
    }

    #[test]
    fn err() {
        let mut buf = [0; _];

        // note: inferred `Result<u32, ()>` from `SerializeBuf` impl in `ok`
        let used = Err(()).serialize_buf(&mut buf);

        assert_eq!(used, 1);
    }
}
