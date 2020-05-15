//!Iterator over byte buffers

use core::{iter};

#[derive(Clone, Copy)]
///Iterator over byte buffer.
pub struct Iter<'a, T> {
    inner: &'a T,
    cursor: usize,
    len: usize,
}

impl<'a, T> Iter<'a, T> {
    #[inline]
    ///Creates new iterator within specified range.
    ///
    ///Range MUST be within [0; T::len()], otherwise behavior depends on index implementation.
    pub const fn new(inner: &'a T, from: usize, to: usize) -> Self {
        Self {
            inner,
            cursor: from,
            len: to,
        }
    }
}

impl<'a, T: crate::Buf> iter::Iterator for Iter<'a, T> {
    type Item = &'a u8;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cursor != self.len {
            let idx = self.cursor;
            self.cursor += 1;
            Some(&self.inner[idx])
        } else {
            None
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = self.len - self.cursor;
        (size, Some(size))
    }

    #[inline]
    fn count(self) -> usize {
        self.len - self.cursor
    }

    #[inline]
    fn last(mut self) -> Option<Self::Item> {
        self.next_back()
    }
}

impl<'a, T: crate::Buf> iter::ExactSizeIterator for Iter<'a, T> {
    #[inline]
    fn len(&self) -> usize {
        self.len - self.cursor
    }
}

impl<'a, T: crate::Buf> iter::DoubleEndedIterator for Iter<'a, T> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.cursor != self.len {
            let idx = self.len;
            self.len -= 1;
            Some(&self.inner[idx])
        } else {
            None
        }
    }
}

impl<'a, T: crate::Buf> iter::FusedIterator for Iter<'a, T> {
}

///Mutable iterator over byte buffer.
pub struct IterMut<'a, T> {
    inner: &'a mut T,
    cursor: usize,
    len: usize,
}

impl<'a, T> IterMut<'a, T> {
    #[inline]
    ///Creates new iterator within specified range.
    ///
    ///Range MUST be within [0; T::len()], otherwise behavior depends on index implementation.
    pub fn new(inner: &'a mut T, from: usize, to: usize) -> Self {
        Self {
            inner,
            cursor: from,
            len: to,
        }
    }
}

impl<'a, T: crate::Buf> iter::Iterator for IterMut<'a, T> {
    type Item = &'a mut u8;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cursor != self.len {
            let idx = self.cursor;
            self.cursor += 1;
            Some(unsafe { &mut *(self.inner[idx] as *mut u8) })
        } else {
            None
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = self.len - self.cursor;
        (size, Some(size))
    }

    #[inline]
    fn count(self) -> usize {
        self.len - self.cursor
    }

    #[inline]
    fn last(mut self) -> Option<Self::Item> {
        self.next_back()
    }
}

impl<'a, T: crate::Buf> iter::ExactSizeIterator for IterMut<'a, T> {
    #[inline]
    fn len(&self) -> usize {
        self.len - self.cursor
    }
}

impl<'a, T: crate::Buf> iter::DoubleEndedIterator for IterMut<'a, T> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.cursor != self.len {
            let idx = self.len;
            self.len -= 1;
            Some(unsafe { &mut *(self.inner[idx] as *mut u8) })
        } else {
            None
        }
    }
}

impl<'a, T: crate::Buf> iter::FusedIterator for IterMut<'a, T> {
}
