use crate::encoding::{Encoding, vanilla::Vanilla};

/// Types implement this trait to be used as serialization mediums.
pub trait Medium<E: Encoding = Vanilla> {
    fn default() -> Self;
}
