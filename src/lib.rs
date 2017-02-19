#![no_std]

extern crate libc;
#[cfg(test)]
extern crate std;

use core::fmt;
use core::ops::Deref;
use core::ptr;
use core::slice;
use core::str::{Utf8Error, self};
use libc::{c_char, c_void};

const DUMMY_PTR: *mut c_void = 0x1 as *mut c_void;

/// A type that represents a `malloc`'d chunk of memory.
pub struct Malloc<T: ?Sized> {
    ptr: *mut T,
}

impl<T> Malloc<T> {
    pub unsafe fn new(ptr: *mut T) -> Malloc<T> {
        Malloc { ptr: ptr }
    }
}

impl<T> Malloc<[T]> {
    /**
    Constructs a new `Malloc` for a `malloc`'d buffer with the given length
    at the given pointer.
    When this `Malloc` drops, the buffer will be `free`'d.

    Unsafe because there must be `len` contiguous, valid instances of `T`
    at `ptr`.

    The given pointer must not be null unless the length is 0; this function
    will specially handle null, 0-length buffers safely.
    */
    pub unsafe fn from_array(ptr: *mut T, len: usize) -> Malloc<[T]> {
        // Even a 0-size slice cannot be null, so just use another pointer
        let ptr = if ptr.is_null() && len == 0 { DUMMY_PTR as *mut T }
                  else { ptr };
        let slice = slice::from_raw_parts(ptr, len);
        Malloc { ptr: slice as *const [T] as *mut [T] }
    }
}

impl Malloc<str> {
    pub unsafe fn from_c_str(ptr: *mut c_char)
            -> Result<Malloc<str>, Utf8Error> {
        let len = libc::strlen(ptr);
        let slice = slice::from_raw_parts(ptr as *mut u8, len);
        str::from_utf8(slice).map(|s| {
            Malloc { ptr: s as *const str as *mut str }
        })
    }
}

impl<T: ?Sized> Deref for Malloc<T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.ptr }
    }
}

impl<T: ?Sized> Drop for Malloc<T> {
    fn drop(&mut self) {
        if (self.ptr as *mut c_void) != DUMMY_PTR {
            unsafe {
                ptr::drop_in_place(self.ptr);
                libc::free(self.ptr as *mut c_void);
            }
        }
    }
}

impl<T: fmt::Debug + ?Sized> fmt::Debug for Malloc<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

impl<T: fmt::Display + ?Sized> fmt::Display for Malloc<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&**self, f)
    }
}

impl<T: ?Sized> AsRef<T> for Malloc<T> {
    fn as_ref(&self) -> &T {
        &**self
    }
}

#[cfg(test)]
mod tests {
    use std::mem;
    use std::ptr;
    use libc::{c_char, self};

    use super::Malloc;

    fn alloc<T>(value: T) -> *mut T {
        unsafe {
            let ptr = libc::malloc(mem::size_of::<T>()) as *mut T;
            ptr::write(ptr, value);
            ptr
        }
    }

    #[test]
    fn test_null_buf() {
        let buf = unsafe {
            Malloc::<[u32]>::from_array(ptr::null_mut(), 0)
        };
        assert!(&*buf == []);
        assert!(Some(&*buf) == Some(&[]));
    }

    #[test]
    fn test_buf() {
        let ptr = alloc([1, 2, 3]);
        let buf = unsafe { Malloc::from_array(ptr as *mut i32, 3) };
        assert!(&*buf == [1, 2, 3]);
    }

    #[test]
    fn test_string() {
        let ptr = alloc(['h' as c_char, 'e' as c_char, 'y' as c_char, '\0' as c_char]);
        let s = unsafe { Malloc::from_c_str(ptr as *mut c_char).unwrap() };
        assert!(&*s == "hey");
    }

    #[test]
    fn test_single() {
        use std::string::ToString;

        let m = unsafe { Malloc::new(alloc(4)) };
        assert!(&*m == &4);

        let m = unsafe { Malloc::new(alloc("hello".to_string())) };
        assert!(&**m == "hello");
    }

    #[test]
    fn test_drop() {
        use std::rc::Rc;

        let num: Rc<i32> = Rc::new(4);
        assert_eq!(Rc::strong_count(&num), 1);

        let ptr = alloc([num.clone(), num.clone()]);
        let buf = unsafe { Malloc::from_array(ptr as *mut Rc<i32>, 2) };
        assert_eq!(Rc::strong_count(&num), 3);

        drop(buf);
        assert_eq!(Rc::strong_count(&num), 1);
    }
}
