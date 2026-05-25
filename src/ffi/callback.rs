#![allow(dead_code)]

/// Callback function called on panic.
/// - `info`: Pointer to the panic information string.
#[unsafe(no_mangle)]
#[linkage = "weak"]
pub unsafe extern "C" fn shitty_callback_panic(_info: *const u8) -> ! {
    #[cfg(feature = "ffi-std")]
    {
        unsafe extern "C" {
            fn abort() -> !;
        }
        unsafe { abort() };
    }
    #[allow(unreachable_code)]
    loop {
        core::hint::spin_loop();
    }
}

unsafe extern "C" {
    /// Callback function called on memory allocation.
    /// - `size`: The size of the memory to allocate.
    pub fn shitty_callback_alloc(size: usize) -> *mut u8;

    /// Callback function called on memory deallocation.
    /// - `ptr`: Pointer to the memory to deallocate.
    pub fn shitty_callback_free(ptr: *mut u8);

    /// Callback function called on aligned memory allocation.
    /// - `size`: The size of the memory to allocate.
    /// - `align`: The alignment of the memory to allocate.
    pub fn shitty_callback_aligned_alloc(size: usize, align: usize) -> *mut u8;

    /// Callback function called on memory reallocation.
    /// - `ptr`: Pointer to the memory to reallocate.
    /// - `size`: The new size of the memory.
    pub fn shitty_callback_realloc(ptr: *mut u8, size: usize) -> *mut u8;

    /// Callback function called on aligned memory reallocation.
    /// - `ptr`: Pointer to the memory to reallocate.
    /// - `size`: The new size of the memory.
    /// - `align`: The alignment of the memory to allocate.
    pub fn shitty_callback_aligned_realloc(ptr: *mut u8, size: usize, align: usize) -> *mut u8;
}
