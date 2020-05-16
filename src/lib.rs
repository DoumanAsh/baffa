//! Generic byte buffer

#![no_std]
#![warn(missing_docs)]
#![cfg_attr(feature = "cargo-clippy", allow(clippy::style))]

#[cfg(feature = "std")]
extern crate std;

use core::{mem, cmp, ops};

pub mod stack;
pub mod iter;

///Alias to static buffer.
pub type StaticBuffer<T> = stack::Buffer<T>;
///Alias to circular buffer.
pub type RingBuffer<T> = stack::Ring<T>;

///Common buffer.
pub trait Buf: ops::IndexMut<usize, Output=u8> + Sized {
    ///Returns size of the underlying memory in the buffer.
    fn capacity(&self) -> usize;

    ///Returns number of elements inside the buffer.
    fn len(&self) -> usize;

    #[inline]
    ///Returns iterator over elements inside the buffer.
    fn iter(&self) -> iter::Iter<'_, Self> {
        iter::Iter::new(self, 0, self.len())
    }

    //TODO: separate unsafe trait? Technically need to beware of IndexMut::index_mut returning the
    //same address
    #[inline]
    ///Returns mutable iterator over elements inside the buffer.
    fn iter_mut(&mut self) -> iter::IterMut<'_, Self> {
        iter::IterMut::new(self, 0, self.len())
    }
}

///Describes buffer that allows to extend capacity
pub trait DynBuf {
    ///Reserves additional space, enough to at least fit `size`.
    ///
    ///Generally should be noop if there is enough capacity.
    fn reserve(&mut self, size: usize);
    ///Removes `size` number of bytes from underlying memory.
    ///
    ///If `size` is bigger than `capacity` should behave as if `size` is equal (i.e. clear whole
    ///memory).
    fn shrink(&mut self, size: usize);
}

///Describes read-able buffer
pub trait ReadBuf: Buf {
    #[inline(always)]
    ///Returns number of bytes left
    ///
    ///Returns buffer's `length` by default
    fn available(&self) -> usize {
        Buf::len(self)
    }

    ///Moves cursor, considering bytes as consumed.
    unsafe fn consume(&mut self, step: usize);

    ///Low level read function, that consumes available bytes up to `size`.
    ///
    ///This function is always used in safe manner by other default implementations:
    ///
    ///- `size` is always `min(buffer_size, available)`
    ///- `ptr` is always non-null.
    unsafe fn read(&mut self, ptr: *mut u8, size: usize);

    #[inline]
    ///Reads available bytes into slice
    fn read_slice(&mut self, bytes: &mut [u8]) -> usize {
        let read_len = cmp::min(bytes.len(), self.available());

        if read_len > 0 {
            unsafe {
                self.read(bytes.as_mut_ptr(), bytes.len())
            }
        }

        read_len
    }
}

///Extension trait to provide extra functionality
pub trait ReadBufExt: ReadBuf {
    #[inline]
    ///Reads value into storage.
    ///
    ///If not enough bytes, does nothing, returning 0
    fn read_value<T: Copy + Sized>(&mut self, val: &mut mem::MaybeUninit<T>) -> usize {
        let size = mem::size_of::<T>();

        if size != 0 && self.available() >= size {
            unsafe {
                self.read(val.as_mut_ptr() as *mut u8, size);
            }
            size
        } else {
            0
        }
    }
}

impl<T: ReadBuf> ReadBufExt for T {}

///Describes write-able buffer
pub trait WriteBuf: Buf {
    #[inline(always)]
    ///Returns number of bytes left
    ///
    ///Default implementation returns `capacity - len`
    fn remaining(&self) -> usize {
        self.capacity() - self.len()
    }

    ///Moves cursor, considering bytes written.
    unsafe fn advance(&mut self, step: usize);

    ///Low level write method, which copies data from pointer up to `size`.
    ///
    ///This function is always used in safe manner by other default implementations:
    ///
    ///- `size` is always `min(buffer_size, available)`
    ///- `ptr` is always non-null.
    unsafe fn write(&mut self, ptr: *const u8, size: usize);

    #[inline]
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

///Extension trait to provide extra functionality
pub trait WriteBufExt: WriteBuf {
    #[inline]
    ///Writes supplied value by performing bit copy, advancing length and returning number of bytes written.
    ///
    ///If value cannot fit, does nothing
    fn write_value<T: Copy + Sized>(&mut self, val: &T) -> usize {
        let size = mem::size_of::<T>();

        if size != 0 && self.remaining() >= size {
            unsafe {
                self.write(val as *const _ as *const u8, size);
            }
            size
        } else {
            0
        }
    }
}

impl<T: WriteBuf> WriteBufExt for T {}
