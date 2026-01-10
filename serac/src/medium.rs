use crate::encoding::{vanilla::Vanilla, Encoding};

// TODO: iters should be associated types defined
// by implementors

/// Types implement this trait to be used
/// as serialization mediums.
pub trait Medium<E: Encoding = Vanilla> {
    fn default() -> Self;
}
