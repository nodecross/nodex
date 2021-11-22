// refs: gist (@huntc) : mod.rs
// https://gist.github.com/huntc/ab9a505683647aac7bccd2df0fc75f9e

extern crate alloc;

use core::{alloc::{GlobalAlloc, Layout}, ptr::null_mut};

/// In the absence of std, we need our own c_void so that cbindgen is happy to
/// generate void*
#[allow(non_camel_case_types)]
pub struct c_void;

/// An allocator that provides the ability to be configured with an outside allocator.
///
/// An external heap is one that exists outside of the one normally used by
/// Rust. For example, when running in an ARM cortex-m environment and with
/// Rust being called as a static library from C, an allocator and deallocator
/// can be provided by the caller. A requirement is that a static `GlobalAllocator`
/// structure is established with the `empty` method.
///
/// ```rust
/// mod external_heap;
/// use external_heap::{c_void, ExternalHeap};
///
/// #[global_allocator]
/// static mut ALLOCATOR: ExternalHeap = ExternalHeap::empty();
///
/// use core::alloc::Layout;
///
/// // Allocations will return a null pointer as `ExternalHeap`'s `init` method
/// // has not been called yet.
/// assert_eq!(
///     unsafe { ALLOCATOR.alloc(Layout::new::<u8>()) } as *mut c_void,
///     0 as *mut c_void
/// )
/// ```
pub struct ExternalHeap {
    allocator  : extern "C" fn(u32) -> *mut c_void,
    deallocator: extern "C" fn(*mut c_void),
}

impl ExternalHeap {
    /// Initialize the static allocator with benign allocation/deallocation.
    /// This will always be the first call to make.
    pub const fn empty() -> ExternalHeap {
        ExternalHeap {
            allocator  : ExternalHeap::noop_allocator,
            deallocator: ExternalHeap::noop_deallocator,
        }
    }

    /// Set up the external allocation/deallocation functions. This should be
    /// done prior to performing any allocations, otherwise you may find that
    /// panics occur given null allocations.
    pub fn init(
        &mut self,
        allocator  : extern "C" fn(u32) -> *mut c_void,
        deallocator: extern "C" fn(*mut c_void),
    ) {
        self.allocator   = allocator;
        self.deallocator = deallocator;
    }

    extern "C" fn noop_allocator(_size: u32) -> *mut c_void {
        null_mut::<c_void>()
    }

    extern "C" fn noop_deallocator(_ptr: *mut c_void) {}
}

unsafe impl GlobalAlloc for ExternalHeap {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        (self.allocator)(layout.size() as u32) as *mut u8
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        (self.deallocator)(ptr as *mut c_void)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    extern "C" fn test_allocator(size: u32) -> *mut c_void {
        assert_eq!(size, 1);

        1 as *mut c_void
    }

    extern "C" fn test_deallocator(ptr: *mut c_void) {
        assert_eq!(ptr, 1 as *mut c_void)
    }

    #[test]
    fn init() {
        static mut ALLOCATOR: ExternalHeap = ExternalHeap::empty();

        use core::alloc::Layout;

        let layout = Layout::new::<u8>();

        // Allocations will return a null pointer as `ExternalHeap`'s `init` method
        // has not been called yet.
        assert_eq!(
            unsafe { ALLOCATOR.alloc(layout) } as *mut c_void,
            null_mut::<c_void>()
        );

        // Now setup an alloc/dealloc handler and assert its usage

        unsafe { ALLOCATOR.init(test_allocator, test_deallocator) };

        let alloc = unsafe { ALLOCATOR.alloc(Layout::new::<u8>()) };

        assert_eq!(alloc as *mut c_void, 1 as *mut c_void);

        unsafe { ALLOCATOR.dealloc(alloc, layout) }
    }
}