//! Generic byte buffer

#![warn(missing_docs)]
#![cfg_attr(feature = "cargo-clippy", allow(clippy::style))]

#[cfg(feature = "std")]
extern crate std;

use core::{ptr, cmp};

pub mod stack;

///Alias to static buffer.
pub type StaticBuffer<T> = stack::Buffer<T>;

///Describes mutable buffer
pub trait WriteBuf: Sized {
    ///Returns number of bytes left
    fn remaining(&mut self) -> usize;
    ///Moves cursor, considering bytes written.
    unsafe fn advance(&mut self, step: usize);
    ///Returns pointer to the first element, that is yet to be written.
    fn cursor(&mut self) -> *mut u8;

    #[inline]
    ///Low level write method, which uses pointer with size.
    ///
    ///This is unsafe because it can assume that `ptr` and `size` are valid.
    unsafe fn write(&mut self, ptr: *const u8, size: usize) {
        ptr::copy_nonoverlapping(ptr, self.cursor(), size);
        self.advance(size);
    }

    ///Writes supplied slice into the buffer, returning number of written bytes.
    ///
    ///Allows partial writes.
    fn write_slice(&mut self, bytes: &[u8]) -> usize {
        let write_len = cmp::min(bytes.len(), self.remaining());

        if write_len > 0 {
            unsafe {
                self.write(bytes.as_ptr(), write_len);
            }
        }

        write_len
    }
}
