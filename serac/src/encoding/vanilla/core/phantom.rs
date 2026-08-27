use core::marker::PhantomData;

use crate::{Buf, Encoding, SerializeBuf, SerializeIter, Size, encoding::Vanilla, error};

impl<T> SerializeIter for PhantomData<T> {
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
        Ok(PhantomData)
    }
}

unsafe impl<T> Size for PhantomData<T> {
    const SIZE: usize = 0;
}

unsafe impl<T> SerializeBuf<0> for PhantomData<T> {}

#[cfg(test)]
mod tests {
    use core::marker::PhantomData;

    use crate::SerializeBuf as _;

    #[test]
    fn zst() {
        let mut buf = [0; _];

        let used = PhantomData::<u64>.serialize_buf(&mut buf);

        assert_eq!(used, 0);
    }
}
