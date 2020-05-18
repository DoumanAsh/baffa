//! Stack based buffer

use core::{cmp, fmt, slice, mem, ptr, ops};
use crate::{Buf, ContBuf, ReadBuf, WriteBuf};

///Static buffer to raw bytes
///
///The size of the storage must be known at compile time, therefore it is suitable only for
///non-dynamic storages.
///
///While write semantics are pretty obvious, read behaviour is more complicated due to it being a
///static buffer.
///
///When performing `ReadBuf::read` memory is always reading from the beginning.
///So as with any other implementation read leads to consumption.
///But as this buffer is single chunk of static memory, such operation will require to shift
///already written bytes to the beginning (meaning each `ReadBuf::consume` involves a `memmove`
///unless consumed `len` is not equal to current `len`)
///
///In general it would be more effective to access memory as slice and then consume it, if needed.
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
    ///Transforms buffer into ring buffer.
    pub const fn into_circular(self) -> Ring<S> {
        unsafe {
            Ring::from_parts(self, 0)
        }
    }

    #[inline]
    ///Creates new instance from parts.
    ///
    ///`cursor` - number of elements written. It is user responsibility to make sure it is not over
    ///actual capacity
    pub const unsafe fn from_parts(inner: mem::MaybeUninit<S>, cursor: usize) -> Self {
        Self {
            inner,
            cursor,
        }
    }

    #[inline]
    ///Splits buffer into parts.
    pub const fn into_parts(self) -> (mem::MaybeUninit<S>, usize) {
        (self.inner, self.cursor)
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

    #[inline]
    ///Returns number of bytes written.
    pub const fn len(&self) -> usize {
        self.cursor
    }
}

impl<S: Sized> ops::Index<usize> for Buffer<S> {
    type Output = u8;

    #[inline(always)]
    fn index(&self, index: usize) -> &Self::Output {
        debug_assert!(index < self.len());
        unsafe {
            &*self.as_ptr().offset(index as isize)
        }
    }
}

impl<S: Sized> ops::IndexMut<usize> for Buffer<S> {
    #[inline(always)]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        debug_assert!(index < self.len());
        unsafe {
            &mut *(self.as_ptr().offset(index as isize) as *mut _)
        }
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

impl<S: Sized> Buf for Buffer<S> {
    #[inline(always)]
    fn capacity(&self) -> usize {
        Self::capacity()
    }

    #[inline(always)]
    fn len(&self) -> usize {
        self.cursor
    }
}

impl<S: Sized> WriteBuf for Buffer<S> {
    #[inline(always)]
    fn remaining(&self) -> usize {
        Self::remaining(self)
    }

    #[inline(always)]
    unsafe fn advance(&mut self, step: usize) {
        self.set_len(self.cursor + step);
    }

    unsafe fn write(&mut self, ptr: *const u8, size: usize) {
        debug_assert!(!ptr.is_null());

        ptr::copy_nonoverlapping(ptr, self.as_ptr().offset(self.cursor as isize) as *mut u8, size);
        self.advance(size);
    }
}

impl<S: Sized> ReadBuf for Buffer<S> {
    unsafe fn consume(&mut self, step: usize) {
        debug_assert!(step <= self.cursor);

        if step == 0 {
            return
        }

        let remaining = self.cursor.saturating_sub(step);

        if remaining != 0 {
            ptr::copy(self.as_ptr().offset(step as isize), self.as_ptr() as *mut u8, remaining);
        }

        self.set_len(remaining)
    }

    unsafe fn read(&mut self, ptr: *mut u8, size: usize) {
        debug_assert!(!ptr.is_null());

        ptr::copy_nonoverlapping(self.as_ptr(), ptr, size);
        self.consume(size);
    }
}

impl<S: Sized> ContBuf for Buffer<S> {
    #[inline(always)]
    fn as_read_slice(&self) -> &[u8] {
        self.as_slice()
    }

    #[inline(always)]
    fn as_read_slice_mut(&mut self) -> &mut [u8] {
        self.as_mut_slice()
    }

    #[inline(always)]
    fn as_write_slice(&mut self) -> &mut [mem::MaybeUninit<u8>] {
        unsafe {
            slice::from_raw_parts_mut(self.as_ptr().offset(self.cursor as isize) as *mut mem::MaybeUninit<u8>, Self::capacity() - self.cursor)
        }
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

///Circular version of `Buffer`
///
///Because `Buffer` becomes circular, it always has remaining bytes to write.
///But care must be taken because without consuming already written bytes, it is easy to over-write
pub struct Ring<T: Sized> {
    buffer: Buffer<T>,
    read: usize
}

impl<S: Sized> Ring<S> {
    #[inline]
    ///Creates new instance
    pub const fn new() -> Self {
        unsafe {
            Self::from_parts(Buffer::new(), 0)
        }
    }

    #[inline]
    ///Creates new instance from parts
    pub const unsafe fn from_parts(buffer: Buffer<S>, read: usize) -> Self {
        Self {
            buffer,
            read
        }
    }

    #[inline]
    ///Creates new instance from parts
    pub const fn into_parts(self) -> (Buffer<S>, usize) {
        (self.buffer, self.read)
    }

    #[inline]
    const fn mask_idx(idx: usize) -> usize {
        idx & (Buffer::<S>::capacity() - 1)
    }

    ///Returns number of available elements
    pub const fn len(&self) -> usize {
        self.buffer.cursor - self.read
    }

    ///Returns whether buffer is empty.
    pub const fn is_empty(&self) -> bool {
        self.buffer.cursor == self.read
    }

    ///Returns whether buffer is full.
    pub const fn is_full(&self) -> bool {
        Buffer::<S>::capacity() == self.len()
    }
}

impl<S: Sized> ops::Index<usize> for Ring<S> {
    type Output = u8;

    #[inline(always)]
    fn index(&self, mut index: usize) -> &Self::Output {
        debug_assert!(index < self.len());
        index = Self::mask_idx(self.read.wrapping_add(index));
        unsafe {
            &*self.buffer.as_ptr().offset(index as isize)
        }
    }
}

impl<S: Sized> ops::IndexMut<usize> for Ring<S> {
    #[inline(always)]
    fn index_mut(&mut self, mut index: usize) -> &mut Self::Output {
        debug_assert!(index < self.len());
        index = Self::mask_idx(self.read.wrapping_add(index));
        unsafe {
            &mut *(self.buffer.as_ptr().offset(index as isize) as *mut _)
        }
    }
}

impl<S: Sized> Buf for Ring<S> {
    #[inline(always)]
    fn capacity(&self) -> usize {
        Buffer::<S>::capacity()
    }

    #[inline(always)]
    fn len(&self) -> usize {
        Self::len(self)
    }
}

impl<S: Sized> ReadBuf for Ring<S> {
    #[inline(always)]
    fn available(&self) -> usize {
        Self::len(self)
    }

    #[inline]
    unsafe fn consume(&mut self, step: usize) {
        self.read = self.read.wrapping_add(step);
    }

    unsafe fn read(&mut self, ptr: *mut u8, mut size: usize) {
        debug_assert!(!ptr.is_null());
        debug_assert!((Buffer::<S>::capacity() & (Buffer::<S>::capacity() - 1)) == 0, "Capacity is not power of 2");
        let idx = Self::mask_idx(self.read);
        let read_span = cmp::min(Buffer::<S>::capacity() - idx, size);

        ptr::copy_nonoverlapping(self.buffer.as_ptr().offset(idx as isize), ptr, read_span);
        self.consume(read_span);
        size -= read_span;

        if size > 0 {
            let avail_size = cmp::min(size, self.available());
            if avail_size > 0 {
                ptr::copy_nonoverlapping(self.buffer.as_ptr(), ptr.offset(read_span as isize), avail_size);
                self.consume(avail_size);
            }
        }
    }
}

impl<S: Sized> WriteBuf for Ring<S> {
    #[inline(always)]
    fn remaining(&self) -> usize {
        Buffer::<S>::capacity()
    }

    #[inline(always)]
    unsafe fn advance(&mut self, step: usize) {
        self.buffer.cursor = self.buffer.cursor.wrapping_add(step);

        let read_span = self.buffer.cursor - self.read;
        if read_span > Buffer::<S>::capacity() {
            //consume over-written bytes
            self.consume(read_span - Buffer::<S>::capacity());
        }
    }

    unsafe fn write(&mut self, ptr: *const u8, mut size: usize) {
        debug_assert!(!ptr.is_null());
        debug_assert!((Buffer::<S>::capacity() & (Buffer::<S>::capacity() - 1)) == 0, "Capacity is not power of 2");

        let cursor = Self::mask_idx(self.buffer.cursor);
        let mut write_span = cmp::min(Buffer::<S>::capacity() - cursor, size);

        ptr::copy_nonoverlapping(ptr, self.buffer.as_ptr().offset(cursor as isize) as *mut u8, write_span);
        size -= write_span;

        while size > 0 {
            let avail_size = cmp::min(size, Buffer::<S>::capacity());

            ptr::copy_nonoverlapping(ptr.offset(write_span as isize), self.buffer.as_ptr() as *mut u8, avail_size);
            size -= avail_size;
            write_span += avail_size;
        }

        self.advance(write_span);
    }
}
