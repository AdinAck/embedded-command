use super::CRCProvider;

/// An iterator wrapper that computes
/// the CRC of the bytes iterated over.
pub(crate) struct CRCComputeIter<'a, 'b, C, I>
where
    C: CRCProvider,
    I: Iterator<Item = &'b C::Word>,
    C::Word: 'b,
{
    crc_provider: &'a mut C,
    iter: &'a mut I,
}

impl<'a, 'b, C, I> CRCComputeIter<'a, 'b, C, I>
where
    C: CRCProvider,
    I: Iterator<Item = &'b C::Word>,
    C::Word: 'b,
{
    pub fn new(crc_provider: &'a mut C, iter: &'a mut I) -> Self {
        Self { crc_provider, iter }
    }
}

impl<'a, 'b, C, I> Iterator for CRCComputeIter<'a, 'b, C, I>
where
    C: CRCProvider,
    I: Iterator<Item = &'b C::Word>,
    C::Word: 'b,
{
    type Item = &'b C::Word;

    fn next(&mut self) -> Option<Self::Item> {
        let word = self.iter.next()?;

        self.crc_provider.update(word);

        Some(word)
    }
}

/// An iterator wrapper that computes
/// the CRC of the mutable bytes iterated over.
pub(crate) struct CRCComputeIterMut<'a, 'b, C, I>
where
    C: CRCProvider,
    I: Iterator<Item = &'b mut C::Word>,
    C::Word: 'b,
{
    crc_provider: &'a mut C,
    iter: &'a mut I,
}

impl<'a, 'b, C, I> CRCComputeIterMut<'a, 'b, C, I>
where
    C: CRCProvider,
    I: Iterator<Item = &'b mut C::Word>,
    C::Word: 'b,
{
    pub fn new(crc_provider: &'a mut C, iter: &'a mut I) -> Self {
        Self { crc_provider, iter }
    }
}

impl<'a, 'b, C, I> Iterator for CRCComputeIterMut<'a, 'b, C, I>
where
    C: CRCProvider,
    I: Iterator<Item = &'b mut C::Word>,
    C::Word: 'b,
{
    type Item = &'b mut C::Word;

    fn next(&mut self) -> Option<Self::Item> {
        let word = self.iter.next()?;

        self.crc_provider.update(word);

        Some(word)
    }
}
