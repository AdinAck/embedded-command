use core::marker::PhantomData;

use cookie_cutter::SerializeIter;
use iter::{CRCComputeIter, CRCComputeIterMut};

mod iter;

#[derive(Debug)]
pub enum Error {
    Serialize(cookie_cutter::error::Error),
    Crc,
}

impl From<cookie_cutter::error::Error> for Error {
    fn from(value: cookie_cutter::error::Error) -> Self {
        Self::Serialize(value)
    }
}

/// Describes types that can provide
/// a CRC computation.
pub trait CRCProvider {
    type Word;
    type Rep: SerializeIter + Eq;

    fn update(&mut self, word: &Self::Word);
    fn finalize(&mut self) -> Self::Rep;
}

/// A packet with an attached CRC
/// for transmission validation.
#[derive(Debug, PartialEq)]
pub struct CRCPacket<P: SerializeIter, C: CRCProvider> {
    payload: P,
    _crc_provider: PhantomData<C>,
}

impl<P: SerializeIter, C: CRCProvider<Word = u8>> CRCPacket<P, C> {
    pub const fn new(payload: P) -> Self {
        Self {
            payload,
            _crc_provider: PhantomData,
        }
    }

    /// Render the packet to bytes for transmission.
    pub fn render<'a>(
        &'a self,
        dst: impl IntoIterator<Item = &'a mut C::Word>,
        crc_provider: &'a mut C,
    ) -> Result<(), cookie_cutter::error::EndOfInput>
    where
        C::Word: 'a,
    {
        let mut dst = dst.into_iter();

        let crc_iter = CRCComputeIterMut::new(crc_provider, &mut dst);
        self.payload.serialize_iter(crc_iter)?;
        let crc = crc_provider.finalize();
        crc.serialize_iter(&mut dst)?;

        Ok(())
    }

    /// Construct the packet from bytes.
    pub fn construct<'a>(
        src: impl IntoIterator<Item = &'a C::Word>,
        crc_provider: &'a mut C,
    ) -> Result<Self, Error> {
        let mut src = src.into_iter();

        let crc_iter = CRCComputeIter::new(crc_provider, &mut src);
        let payload = P::deserialize_iter(crc_iter)?;
        let computed_crc = crc_provider.finalize();
        let read_crc = C::Rep::deserialize_iter(src)?;

        if computed_crc != read_crc {
            Err(Error::Crc)?
        }

        Ok(Self {
            payload,
            _crc_provider: PhantomData,
        })
    }
}

#[cfg(test)]
mod tests {
    use cookie_cutter::encoding::vanilla;

    use super::*;

    #[derive(Debug, PartialEq)]
    struct DummyCRC;

    impl CRCProvider for DummyCRC {
        type Word = u8;
        type Rep = u16;

        fn update(&mut self, _word: &Self::Word) {}

        fn finalize(&mut self) -> Self::Rep {
            0xbeef
        }
    }

    #[derive(Debug, PartialEq, vanilla::SerializeIter, vanilla::SerializeBuf)]
    struct Foo {
        a: i8,
        b: u32,
    }

    #[test]
    fn basic() {
        let mut buf = [0u8; 7];

        let test_packet = CRCPacket::new(Foo {
            a: -1,
            b: 0xdeadbeef,
        });

        let mut crc_provider = DummyCRC;

        assert!(test_packet
            .render(buf.iter_mut(), &mut crc_provider)
            .is_ok());

        assert_eq!(
            buf,
            [-1i8 as u8, 0xef, 0xbe, 0xad, 0xde, /* crc -> */ 0xef, 0xbe]
        );

        let read_packet = CRCPacket::construct(buf.iter(), &mut crc_provider).unwrap();

        assert_eq!(test_packet, read_packet);
    }

    #[test]
    fn bad_crc() {
        let mut buf = [0u8; 7];

        let test_packet = CRCPacket::new(Foo {
            a: -1,
            b: 0xdeadbeef,
        });

        let mut crc_provider = DummyCRC;

        assert!(test_packet
            .render(buf.iter_mut(), &mut crc_provider)
            .is_ok());

        // bad crc
        buf[6] += 1;

        match CRCPacket::<Foo, _>::construct(buf.iter(), &mut crc_provider) {
            Err(e) => match e {
                Error::Crc => {}
                _ => panic!("Error should be CRC mismatch."),
            },
            _ => panic!("CRCPacket construction should fail."),
        }
    }
}
