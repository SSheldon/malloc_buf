#![feature(unsafe_destructor)]

extern crate libc;

use std::ffi::CStr;
use std::ops::Deref;
use std::ptr;
use std::slice;
use std::str;
use libc::{c_char, c_void};

/// A type that represents a `malloc`'d chunk of memory.
pub struct MallocBuffer<T> {
    ptr: *mut T,
    len: usize,
}

impl<T> MallocBuffer<T> {
    /// Constructs a new `MallocBuffer` for a `malloc`'d buffer
    /// with the given length at the given pointer.
    /// Returns `None` if the given pointer is null and the length is not 0.
    ///
    /// When this `MallocBuffer` drops, the elements of the buffer will be
    /// dropped and the buffer will be `free`'d.
    ///
    /// Unsafe because there must be `len` contiguous, valid instances of `T`
    /// at `ptr`.
    pub unsafe fn new(ptr: *mut T, len: usize) -> Option<MallocBuffer<T>> {
        if len > 0 && ptr.is_null() {
            None
        } else {
            Some(MallocBuffer { ptr: ptr, len: len })
        }
    }
}

#[unsafe_destructor]
impl<T> Drop for MallocBuffer<T> {
    fn drop(&mut self) {
        unsafe {
            for item in self.iter() {
                ptr::read(item);
            }
            libc::free(self.ptr as *mut c_void);
        }
    }
}

impl<T> Deref for MallocBuffer<T> {
    type Target = [T];

    fn deref(&self) -> &[T] {
        let ptr = if self.len == 0 && self.ptr.is_null() {
            // Even a 0-size slice cannot be null, so just use another pointer
            0x1 as *const T
        } else {
            self.ptr as *const T
        };
        unsafe {
            slice::from_raw_parts(ptr, self.len)
        }
    }
}

/// A type that represents a `malloc`'d string.
pub struct MallocString {
    data: MallocBuffer<u8>,
}

impl MallocString {
    /// Constructs a new `MallocString` for a `malloc`'d C string buffer.
    /// Returns `None` if the given pointer is null or the C string isn't UTF8.
    /// When this `MallocString` drops, the buffer will be `free`'d.
    ///
    /// Unsafe because `ptr` must point to a valid, nul-terminated C string.
    pub unsafe fn new(ptr: *mut c_char) -> Option<MallocString> {
        if ptr.is_null() {
            None
        } else {
            let s = CStr::from_ptr(ptr as *const c_char);
            let bytes = s.to_bytes();
            if str::from_utf8(bytes).is_ok() {
                let data = MallocBuffer {
                    ptr: ptr as *mut u8,
                    // len + 1 to account for the nul byte
                    len: bytes.len() + 1,
                };
                Some(MallocString { data: data })
            } else {
                None
            }
        }
    }
}

impl Deref for MallocString {
    type Target = str;

    fn deref(&self) -> &str {
        let v = &self.data[..self.data.len - 1];
        unsafe {
            str::from_utf8_unchecked(v)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::ptr;
    use libc::{c_char, self};

    use super::{MallocBuffer, MallocString};

    #[test]
    fn test_null_buf() {
        let buf = unsafe {
            MallocBuffer::<u32>::new(ptr::null_mut(), 0).unwrap()
        };
        assert!(&*buf == []);
        assert!(Some(&*buf) == Some(&[]));

        let buf = unsafe {
            MallocBuffer::<u32>::new(ptr::null_mut(), 7)
        };
        assert!(buf.is_none());
    }

    #[test]
    fn test_buf() {
        let buf = unsafe {
            let ptr = libc::malloc(12) as *mut u32;
            *ptr = 1;
            *ptr.offset(1) = 2;
            *ptr.offset(2) = 3;
            MallocBuffer::new(ptr, 3).unwrap()
        };
        assert!(&*buf == [1, 2, 3]);
    }

    #[test]
    fn test_string() {
        let s = unsafe {
            let ptr = libc::malloc(4) as *mut c_char;
            *ptr = 'h' as c_char;
            *ptr.offset(1) = 'e' as c_char;
            *ptr.offset(2) = 'y' as c_char;
            *ptr.offset(3) = '\0' as c_char;
            MallocString::new(ptr).unwrap()
        };
        assert!(&*s == "hey");
    }
}
