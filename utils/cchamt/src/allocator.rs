extern crate libc;
use std::mem;
use std::sync::atomic::{AtomicUsize, Ordering};

pub struct Allocator<T> {
    buf: *mut T,
    capacity: usize,
    n: AtomicUsize,
}

impl<T> Allocator<T> {
    pub fn new(size: usize) -> Self {
        Allocator {
            buf: unsafe {
                libc::calloc(size as libc::size_t, mem::size_of::<T>() as libc::size_t) as *mut T
            },
            capacity: size,
            n: AtomicUsize::new(0),
        }
    }

    pub fn alloc(&self, obj: T) -> &mut T {
        let i = self.n.fetch_add(1, Ordering::Relaxed);
        assert!(i < self.capacity);
        unsafe {
            *self.buf.offset(i as isize) = obj;
        }
        unsafe { &mut *self.buf.offset(i as isize) }
    }
}
