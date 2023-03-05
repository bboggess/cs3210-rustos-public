/// Align `addr` downwards to the nearest multiple of `align`.
///
/// The returned usize is always <= `addr.`
///
/// # Panics
///
/// Panics if `align` is not a power of 2.
pub fn align_down(addr: usize, align: usize) -> usize {
    assert!(
        align.is_power_of_two(),
        "align_down: expected alignment {} to be a power of 2",
        align
    );

    // Multiple of a power of 2 means that we should clear out
    // the first log_2(align) bits. align - 1 gives us a number
    // which is 1 exactly in the first log_2(align) bits.
    !(align - 1) & addr
}

/// Align `addr` upwards to the nearest multiple of `align`.
///
/// The returned `usize` is always >= `addr.`
///
/// # Panics
///
/// Panics if `align` is not a power of 2
/// or aligning up overflows the address.
pub fn align_up(addr: usize, align: usize) -> usize {
    assert!(
        align.is_power_of_two(),
        "align_up: expected alignment {} to be a power of 2",
        align
    );

    // There's also a bit fiddling approach to this, but it is not appreciably faster.
    let to_add = (align - (addr % align)) % align;

    match addr.overflowing_add(to_add) {
        (n, false) => n,
        (_, true) => panic!(
            "align_up: overflow: could not align address {} up to {}",
            addr, align
        ),
    }
}
