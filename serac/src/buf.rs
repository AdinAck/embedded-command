/// The buffer operated on to perform serialization or deserialization. This tracks the number of medium elements used.
pub struct Buf<I> {
    iter: I,
    pub(super) used: usize,
}

impl<I> From<I> for Buf<I::IntoIter>
where
    I: IntoIterator,
{
    fn from(iter: I) -> Self {
        Self {
            iter: iter.into_iter(),
            used: 0,
        }
    }
}

impl<I> Iterator for Buf<I>
where
    I: Iterator,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().inspect(|_| self.used += 1)
    }
}
