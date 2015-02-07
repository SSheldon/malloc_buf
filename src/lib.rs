#![feature(core, std_misc, unsafe_destructor)]

extern crate libc;

use std::ffi;
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
    /// Returns `None` if the given pointer is null.
    ///
    /// When this `MallocBuffer` drops, the elements of the buffer will be
    /// dropped and the buffer will be `free`'d.
    ///
    /// Unsafe because there must be `len` contiguous, valid instances of `T`
    /// at `ptr`.
    pub unsafe fn new(ptr: *mut T, len: usize) -> Option<MallocBuffer<T>> {
        if ptr.is_null() {
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
        unsafe {
            slice::from_raw_parts(self.ptr as *const T, self.len)
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
            let const_ptr = ptr as *const c_char;
            let bytes = ffi::c_str_to_bytes(&const_ptr);
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
