use crate::encoding::{vanilla::Vanilla, Encoding};

// TODO: iters should be associated types defined
// by implementors

/// Types implement this trait to be used
/// as serialization mediums.
pub trait Medium<E: Encoding = Vanilla> {
    const SIZE: usize;

    fn get_iter<'a>(&'a self) -> impl Iterator<Item = &'a E::Word>
    where
        E::Word: 'a;
    fn get_iter_mut<'a>(&'a mut self) -> impl Iterator<Item = &'a mut E::Word>
    where
        E::Word: 'a;
}

// Implement `Medium` for all arrays.
impl<E: Encoding, const N: usize> Medium<E> for [E::Word; N] {
    const SIZE: usize = N;

    fn get_iter<'a>(&'a self) -> impl Iterator<Item = &'a E::Word>
    where
        E::Word: 'a,
    {
        self.iter()
    }

    fn get_iter_mut<'a>(&'a mut self) -> impl Iterator<Item = &'a mut E::Word>
    where
        E::Word: 'a,
    {
        self.iter_mut()
    }
}
