use core::mem::MaybeUninit;

pub mod error {
    #[derive(Debug)]
    #[cfg_attr(feature = "defmt", derive(defmt::Format))]
    pub struct Overflow;
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CommandBuffer<const N: usize> {
    buf: [MaybeUninit<u8>; N],
    start_cursor: usize,
    size: usize,
}

impl<const N: usize> Default for CommandBuffer<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> CommandBuffer<N> {
    pub const fn new() -> Self {
        Self {
            buf: [MaybeUninit::uninit(); N],
            start_cursor: 0,
            size: 0,
        }
    }

    /// Ingest incoming partial command bytes.
    pub fn ingest<'a>(
        &mut self,
        src: impl IntoIterator<Item = &'a u8>,
    ) -> Result<(), error::Overflow> {
        let mut src = src.into_iter();

        src.try_for_each(|&byte| {
            if self.len() >= N {
                Err(error::Overflow)?;
            }

            let write_cursor = self.end_cursor();
            // SAFETY:
            // 1. cursor must be next empty value in buf
            unsafe { self.buf.get_unchecked_mut(write_cursor) }.write(byte);
            self.size += 1;

            Ok(())
        })
    }

    /// Evict the provided number of bytes (oldest).
    ///
    /// # Safety
    ///
    /// A value of `count` greater than the number of
    /// values present in the buffer will result in UB.
    unsafe fn evict_unchecked(&mut self, count: usize) {
        self.start_cursor = Self::wrap(self.start_cursor + count);
        self.size -= count;
    }

    /// Wrap a provided cursor to adhere
    /// to the buffer size.
    #[inline]
    fn wrap(cursor: usize) -> usize {
        cursor % N
    }

    /// Get the position of the end of
    /// the populated region of the buffer.
    #[inline]
    fn end_cursor(&self) -> usize {
        Self::wrap(self.start_cursor + self.len())
    }

    /// Get the capacity (maximum length) of the buffer.
    #[inline]
    pub fn capacity(&self) -> usize {
        N
    }

    /// Get the current length of the buffer.
    #[inline]
    pub fn len(&self) -> usize {
        self.size
    }

    /// Determines whether the buffer
    /// is empty or not.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Create an iterator for the command buffer.
    #[inline]
    pub fn iter(&mut self) -> CommandBufferIter<'_, N> {
        CommandBufferIter::new(self)
    }

    /// Evict all bytes that have been visited by
    /// the memented iterator.
    #[inline]
    pub fn flush(&mut self, IterMemento(count): IterMemento) {
        // SAFETY: relies on soundness of implementation of
        // `Iterator::next`.
        // self.count must always be <= parent.size()
        unsafe {
            self.evict_unchecked(count);
        }
    }
}

/// An opaque type that may be created
/// upon the death of a `CommandIter`
/// in order to utilize the final state.
pub struct IterMemento(usize);

struct Counter {
    count: usize,
}

impl Counter {
    const fn new() -> Self {
        Self { count: 0 }
    }

    /// Increment the counter by 1.
    #[inline]
    fn increment(&mut self) {
        self.count += 1;
    }

    /// Get the current value of the counter.
    #[inline]
    fn read(&self) -> &usize {
        &self.count
    }
}

/// The iterator type for `CommandBuffer`.
pub struct CommandBufferIter<'a, const N: usize> {
    parent: &'a CommandBuffer<N>,
    counter: Counter,
}

impl<'a, const N: usize> CommandBufferIter<'a, N> {
    fn new(parent: &'a mut CommandBuffer<N>) -> Self {
        Self {
            parent,
            counter: Counter::new(),
        }
    }

    /// Gets the current cursor position.
    #[inline]
    fn cursor(&self) -> usize {
        CommandBuffer::<N>::wrap(self.parent.start_cursor + self.counter.read())
    }

    /// Determines whether the current cursor position
    /// is valid or not.
    #[inline]
    fn cursor_is_valid(&self) -> bool {
        self.counter.read() < &self.parent.len()
    }

    /// Move the cursor to the next position. A call to this
    /// method should follow a call to `read_unchecked`.
    ///
    ///
    /// Note: If the parity of calls between these two functions
    /// is not 1:1 the behavior will be incorrect but still
    /// sound (no UB).
    #[inline]
    fn move_cursor(&mut self) {
        self.counter.increment();
    }

    /// Read the value of the buffer where the cursor
    /// is pointing.
    ///
    /// # Safety
    ///
    /// This function is safe to call
    /// as long as the cursor is "valid".
    /// This can be ensured with the use of
    /// `cursor_is_valid`.
    #[inline]
    unsafe fn read_unchecked(&self) -> &'a u8 {
        // SAFETY: relies on soundness of implementation of `CommandBuffer`
        // 1. cursor must always be <= N
        // 2. cursor must start at parent's start cursor
        // 3. count must not exceed parent length (proven above)
        self.parent
            .buf
            .get_unchecked(self.cursor())
            .assume_init_ref()
    }

    /// Read the value of the buffer where the cursor
    /// is pointing and move the cursor.
    ///
    /// If the cursor is not valid, returns `None`.
    fn read(&mut self) -> Option<&'a u8> {
        let result = self
            .cursor_is_valid()
            // SAFETY: cursor is valid ^
            .then_some(unsafe { self.read_unchecked() })?;

        self.move_cursor();

        Some(result)
    }

    /// Capture the end state of this iterator
    /// as a memento that may be used by the
    /// `CommandBuffer` to perform special operations.
    #[inline]
    pub fn capture(self) -> IterMemento {
        IterMemento(*self.counter.read())
    }
}

impl<'a, const N: usize> Iterator for CommandBufferIter<'a, N> {
    type Item = &'a u8;

    fn next(&mut self) -> Option<Self::Item> {
        self.read()
    }
}

#[cfg(test)]
mod tests {
    use super::CommandBuffer;

    mod ingestion {
        use super::*;

        #[test]
        fn basic() {
            let mut cmd_buf = CommandBuffer::<10>::new();

            let test_buf = [0xde, 0xad, 0xbe, 0xef];

            cmd_buf.ingest(test_buf.iter()).unwrap();

            assert_eq!(cmd_buf.len(), test_buf.len());

            let mut buf_iter = cmd_buf.iter();

            test_buf
                .iter()
                .zip(&mut buf_iter)
                .for_each(|(left, right)| assert_eq!(left, right));

            assert!(buf_iter.next().is_none());
        }

        #[test]
        fn exact() {
            let mut cmd_buf = CommandBuffer::<10>::new();

            let test_buf = [0xde, 0xad, 0xbe, 0xef, 0x15, 0xba, 0xdb, 0xad, 0xf0, 0x0d];

            cmd_buf.ingest(test_buf.iter()).unwrap();

            assert_eq!(cmd_buf.len(), test_buf.len());

            let mut buf_iter = cmd_buf.iter();

            test_buf
                .iter()
                .zip(&mut buf_iter)
                .for_each(|(left, right)| assert_eq!(left, right));

            assert!(buf_iter.next().is_none());
        }

        #[test]
        fn overflow() {
            let mut cmd_buf = CommandBuffer::<8>::new();

            let test_buf = [0xde, 0xad, 0xbe, 0xef, 0x15, 0xba, 0xdb, 0xad, 0xf0, 0x0d];

            assert!(cmd_buf.ingest(test_buf.iter()).is_err());

            assert_eq!(cmd_buf.len(), cmd_buf.capacity());
        }
    }

    mod iter {
        use super::*;

        #[test]
        fn flush() {
            let mut cmd_buf = CommandBuffer::<10>::new();

            let test_buf = [0xde, 0xad, 0xbe, 0xef];

            cmd_buf.ingest(test_buf.iter()).unwrap();

            assert_eq!(cmd_buf.len(), test_buf.len());

            let mut buf_iter = cmd_buf.iter();

            test_buf
                .iter()
                .zip(&mut buf_iter)
                .for_each(|(left, right)| assert_eq!(left, right));

            assert!(buf_iter.next().is_none());

            let memento = buf_iter.capture();

            cmd_buf.flush(memento);

            assert_eq!(0, cmd_buf.len());

            let test_buf = [0x15, 0xba, 0xdb, 0xad, 0xf0, 0x0d];

            cmd_buf.ingest(test_buf.iter()).unwrap();

            assert_eq!(6, cmd_buf.len());
        }

        #[test]
        fn cycle() {
            let mut cmd_buf = CommandBuffer::<10>::new();

            let test_buf = [0xde, 0xad, 0xbe, 0xef];

            for _ in 0..10 {
                cmd_buf.ingest(test_buf.iter()).unwrap();

                assert_eq!(cmd_buf.len(), test_buf.len());

                let mut buf_iter = cmd_buf.iter();

                test_buf
                    .iter()
                    .zip(&mut buf_iter)
                    .for_each(|(left, right)| assert_eq!(left, right));

                assert!(buf_iter.next().is_none());

                let memento = buf_iter.capture();

                cmd_buf.flush(memento);

                assert_eq!(0, cmd_buf.len());
            }
        }
    }
}
