//! Stack based buffer

use core::{fmt, slice, mem};
use crate::WriteBuf;

///Static buffer to raw bytes
///
///The size of the storage must be known at compile time, therefore it is suitable only for
///non-dynamic storages
pub struct Buffer<T: Sized> {
    inner: mem::MaybeUninit<T>,
    cursor: usize, //number of bytes written
}

impl<S: Sized> Buffer<S> {
    #[inline]
    ///Creates new instance
    pub const fn new() -> Self {
        Self {
            inner: mem::MaybeUninit::uninit(),
            cursor: 0,
        }
    }

    #[inline]
    ///Creates new instance from raw parts.
    ///
    ///`cursor` - elements before it must be written before.
    pub const unsafe fn from_raw_parts(inner: mem::MaybeUninit<S>, cursor: usize) -> Self {
        Self {
            inner,
            cursor,
        }
    }

    #[inline]
    ///Returns pointer  to the beginning of underlying buffer
    pub const fn as_ptr(&self) -> *const u8 {
        &self.inner as *const _ as *const u8
    }

    #[inline]
    ///Returns number of bytes left (not written yet)
    pub const fn remaining(&self) -> usize {
        Self::capacity() - self.cursor
    }

    #[inline]
    ///Returns slice to already written data.
    pub fn as_slice(&self) -> &[u8] {
        unsafe {
            slice::from_raw_parts(self.as_ptr(), self.cursor)
        }
    }

    #[inline]
    ///Returns mutable slice to already written data.
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        unsafe {
            slice::from_raw_parts_mut(self.as_ptr() as *mut u8, self.cursor)
        }
    }

    #[inline]
    ///Shortens the buffer.
    ///
    ///Does nothing if new `cursor` is after current position.
    pub fn truncate(&mut self, cursor: usize) {
        if cursor < self.cursor {
            self.cursor = cursor
        }
    }

    #[inline]
    ///Changes written length, without writing.
    ///
    ///When used, user must guarantee that these bytes are written.
    pub unsafe fn set_len(&mut self, cursor: usize) {
        debug_assert!(cursor <= Self::capacity());
        self.cursor = cursor
    }

    #[inline]
    ///Returns buffer overall capacity.
    pub const fn capacity() -> usize {
        mem::size_of::<S>()
    }
}

impl<S: Sized> AsRef<[u8]> for Buffer<S> {
    #[inline(always)]
    fn as_ref(&self) -> &[u8] {
        self.as_slice()
    }
}

impl<S: Sized> fmt::Debug for Buffer<S> {
    #[inline(always)]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_list().entries(self.as_slice().iter()).finish()
    }
}

impl<S: Sized> WriteBuf for Buffer<S> {
    #[inline(always)]
    fn remaining(&mut self) -> usize {
        Self::remaining(self as &Self)
    }

    #[inline(always)]
    unsafe fn advance(&mut self, step: usize) {
        self.set_len(self.cursor + step);
    }

    #[inline(always)]
    fn cursor(&mut self) -> *mut u8 {
        self.as_ptr() as *mut u8
    }
}

#[cfg(feature = "std")]
impl<S: Sized> std::io::Write for Buffer<S> {
    #[inline(always)]
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        Ok(self.write_slice(buf))
    }

    #[inline(always)]
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
