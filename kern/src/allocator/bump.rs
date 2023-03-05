use core::alloc::Layout;
use core::ptr;

use crate::allocator::util::*;
use crate::allocator::LocalAlloc;

/// A "bump" allocator: allocates memory by bumping a pointer; never frees.
#[derive(Debug)]
pub struct Allocator {
    current: usize,
    end: usize,
}

impl Allocator {
    /// Creates a new bump allocator that will allocate memory from the region
    /// starting at address `start` and ending at address `end`.
    #[allow(dead_code)]
    pub fn new(start: usize, end: usize) -> Allocator {
        Allocator {
            current: start,
            end,
        }
    }
}

impl LocalAlloc for Allocator {
    /// Allocates memory. Returns a pointer meeting the size and alignment
    /// properties of `layout.size()` and `layout.align()`.
    ///
    /// If this method returns an `Ok(addr)`, `addr` will be non-null address
    /// pointing to a block of storage suitable for holding an instance of
    /// `layout`. In particular, the block will be at least `layout.size()`
    /// bytes large and will be aligned to `layout.align()`. The returned block
    /// of storage may or may not have its contents initialized or zeroed.
    ///
    /// # Safety
    ///
    /// The _caller_ must ensure that `layout.size() > 0` and that
    /// `layout.align()` is a power of two. Parameters not meeting these
    /// conditions may result in undefined behavior.
    ///
    /// # Errors
    ///
    /// Returning null pointer (`core::ptr::null_mut`)
    /// indicates that either memory is exhausted
    /// or `layout` does not meet this allocator's
    /// size or alignment constraints.
    unsafe fn alloc(&mut self, layout: Layout) -> *mut u8 {
        // prevent undefined behavior from badly constructed Layout
        if layout.size() == 0 || !layout.align().is_power_of_two() {
            return ptr::null_mut();
        }

        let start_addr = align_up(self.current, layout.align());
        let new_cur = start_addr.saturating_add(layout.size());
        let space_allocated = new_cur.saturating_sub(start_addr);

        // handle out of memory
        if new_cur > self.end || space_allocated < layout.size() {
            return ptr::null_mut();
        }

        self.current = new_cur;

        start_addr as *mut u8
    }

    /// Deallocates the memory referenced by `ptr`.
    ///
    /// # Safety
    ///
    /// The _caller_ must ensure the following:
    ///
    ///   * `ptr` must denote a block of memory currently allocated via this
    ///     allocator
    ///   * `layout` must properly represent the original layout used in the
    ///     allocation call that returned `ptr`
    ///
    /// Parameters not meeting these conditions may result in undefined
    /// behavior.
    unsafe fn dealloc(&mut self, _ptr: *mut u8, _layout: Layout) {
        // LEAKED
    }
}