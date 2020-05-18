extern crate alloc;

use crate::{Buf, ReadBuf, WriteBuf, ContBuf};

use core::{slice, mem, ptr};
use alloc::vec::Vec;

impl Buf for Vec<u8> {
    #[inline(always)]
    fn capacity(&self) -> usize {
        Vec::capacity(self)
    }

    #[inline(always)]
    fn len(&self) -> usize {
        Vec::len(self)
    }
}

impl ContBuf for Vec<u8> {
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
            slice::from_raw_parts_mut(self.as_ptr().offset(self.len() as isize) as *mut mem::MaybeUninit<u8>, self.capacity() - self.len())
        }
    }
}

impl ReadBuf for Vec<u8> {
    unsafe fn consume(&mut self, step: usize) {
        debug_assert!(step <= self.len());

        if step == 0 {
            return
        }

        let remaining = self.len().saturating_sub(step);

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

impl WriteBuf for Vec<u8> {
    #[inline(always)]
    unsafe fn advance(&mut self, step: usize) {
        self.set_len(self.len() + step);
    }

    unsafe fn write(&mut self, ptr: *const u8, size: usize) {
        debug_assert!(!ptr.is_null());

        ptr::copy_nonoverlapping(ptr, self.as_ptr().offset(self.len() as isize) as *mut u8, size);
        self.advance(size);
    }
}
