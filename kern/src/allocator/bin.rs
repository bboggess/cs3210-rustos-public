use core::alloc::Layout;
use core::fmt;
use core::ptr;

use crate::allocator::bump;
use crate::allocator::linked_list::LinkedList;
use crate::allocator::util::*;
use crate::allocator::LocalAlloc;

/// A simple allocator that allocates based on size classes.
///   bin 0 (2^3 bytes)    : handles allocations in (0, 2^3]
///   bin 1 (2^4 bytes)    : handles allocations in (2^3, 2^4]
///   ...
///   bin 29 (2^22 bytes): handles allocations in (2^31, 2^32]
///   
///   map_to_bin(size) -> k
///   

pub struct Allocator {
    /// Fallback allocator when there are no free slots in the requested bin
    global_pool: bump::Allocator,
    bins: [LinkedList; SIZES.len()],
}

/// The size of the memory blocks that each bin handles
const SIZES: [usize; 14] = [
    1 << 3,
    1 << 4,
    1 << 5,
    1 << 6,
    1 << 7,
    1 << 8,
    1 << 9,
    1 << 10,
    1 << 11,
    1 << 12,
    1 << 13,
    1 << 14,
    1 << 15,
    1 << 16,
];

impl Allocator {
    /// Creates a new bin allocator that will allocate memory from the region
    /// starting at address `start` and ending at address `end`.
    pub fn new(start: usize, end: usize) -> Self {
        let bins = [LinkedList::new(); SIZES.len()];
        let global_pool = bump::Allocator::new(start, end);

        Self { global_pool, bins }
    }

    /// Allocates memory that is guaranteed to fit in the `bin_num`th bin.
    /// If there is no free memory in the bin, will get from the fallback allocator.
    ///
    /// It is assumed that you have already checked that `bin_num` is a valid bin
    /// number.
    unsafe fn alloc_from_bin(&mut self, bin_num: usize, layout: Layout) -> *mut u8 {
        let align = layout.align();

        // Can't just take the first available block because the alignment might be wrong
        match self.bins[bin_num]
            .iter_mut()
            .find(|node| node.value() as usize % align == 0)
        {
            Some(node) => node.pop() as *mut u8,
            None => {
                let size = SIZES[bin_num];
                let layout = Layout::from_size_align(size, align).unwrap();

                self.alloc_from_fallback(layout)
            }
        }
    }

    /// Requests memory from the fallback allocator. To be called when there is no free
    /// memory we can take from a bin.
    unsafe fn alloc_from_fallback(&mut self, layout: Layout) -> *mut u8 {
        self.global_pool.alloc(layout)
    }
}

/// Given a request for `size` bytes of memory, determines the appropriate bin number
/// to request from. Returns `None` if there is no bin that can handle requests for `size`
/// bytes (e.g. if `size` is larger than the largest bin)
fn map_to_bin(layout: Layout) -> Option<usize> {
    // Make sure that every block in each bin has the same alignment for easy
    // allocation. We do this by aligning each bin according to the block size it holds.
    let size = core::cmp::max(layout.size(), layout.align());

    // Rather than iterating through each bucket size, take advantage of the fact that
    // all iterations are powers of two. Calculate the actual request size (next_power_of_two),
    // and then the bin index is (roughly) the number of trailing zeros.
    size.checked_next_power_of_two()
        .map(|n| n.trailing_zeros().saturating_sub(3) as usize) // we know this fits in a usize
        .and_then(|n| if n < SIZES.len() { Some(n) } else { None })
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

        match map_to_bin(layout) {
            Some(n) => self.alloc_from_bin(n, layout),
            None => self.alloc_from_fallback(layout),
        }
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
    unsafe fn dealloc(&mut self, ptr: *mut u8, layout: Layout) {
        match map_to_bin(layout) {
            Some(n) => {
                assert!(
                    SIZES[n] >= core::mem::size_of::<usize>(),
                    "dealloc: Freed bin not large enough to hold pointer to next free chunk"
                );

                self.bins[n].push(ptr as *mut usize);
            }
            None => self.global_pool.dealloc(ptr, layout),
        }
    }
}

impl fmt::Debug for Allocator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.bins.iter()).finish()
    }
}
