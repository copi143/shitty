use core::alloc::{GlobalAlloc, Layout};

#[allow(unused_imports)]
use crate::ffi::callback::*;

/// Callback function called on memory allocation.
/// - `size`: The size of the memory to allocate.
#[unsafe(no_mangle)]
#[linkage = "weak"]
#[cfg(feature = "ffi-std")]
pub unsafe extern "C" fn shitty_callback_alloc(size: usize) -> *mut u8 {
    unsafe extern "C" {
        fn malloc(size: usize) -> *mut u8;
    }
    unsafe { malloc(size) }
}

/// Callback function called on memory deallocation.
/// - `ptr`: Pointer to the memory to deallocate.
#[unsafe(no_mangle)]
#[linkage = "weak"]
#[cfg(feature = "ffi-std")]
pub unsafe extern "C" fn shitty_callback_free(ptr: *mut u8) {
    unsafe extern "C" {
        fn free(ptr: *mut u8);
    }
    unsafe { free(ptr) }
}

/// Callback function called on aligned memory allocation.
/// - `size`: The size of the memory to allocate.
/// - `align`: The alignment of the memory to allocate.
#[unsafe(no_mangle)]
#[linkage = "weak"]
#[cfg(feature = "ffi-std")]
pub unsafe extern "C" fn shitty_callback_aligned_alloc(size: usize, align: usize) -> *mut u8 {
    unsafe extern "C" {
        fn aligned_alloc(align: usize, size: usize) -> *mut u8;
    }
    unsafe { aligned_alloc(align, size) }
}

/// Callback function called on memory reallocation.
/// - `ptr`: Pointer to the memory to reallocate.
/// - `size`: The new size of the memory.
#[unsafe(no_mangle)]
#[linkage = "weak"]
pub unsafe extern "C" fn shitty_callback_realloc(ptr: *mut u8, size: usize) -> *mut u8 {
    #[cfg(feature = "ffi-std")]
    {
        unsafe extern "C" {
            fn realloc(ptr: *mut u8, size: usize) -> *mut u8;
        }
        return unsafe { realloc(ptr, size) };
    }
    #[allow(unreachable_code)]
    let new_ptr = unsafe { shitty_callback_alloc(size) };
    if new_ptr.is_null() {
        return ptr;
    }
    unsafe {
        core::ptr::copy_nonoverlapping(ptr, new_ptr, size);
        shitty_callback_free(ptr);
    }
    new_ptr
}

/// Callback function called on aligned memory reallocation.
/// - `ptr`: Pointer to the memory to reallocate.
/// - `size`: The new size of the memory.
/// - `align`: The alignment of the memory to allocate.
#[unsafe(no_mangle)]
#[linkage = "weak"]
pub unsafe extern "C" fn shitty_callback_aligned_realloc(ptr: *mut u8, size: usize, align: usize) -> *mut u8 {
    let new_ptr = unsafe { shitty_callback_aligned_alloc(size, align) };
    if new_ptr.is_null() {
        return ptr;
    }
    unsafe {
        core::ptr::copy_nonoverlapping(ptr, new_ptr, size);
        shitty_callback_free(ptr);
    }
    new_ptr
}

/// 使用 C 回调函数（`shitty_callback_*`）的全局分配器。
///
/// A global allocator using C callback functions (`shitty_callback_*`).
#[allow(dead_code)]
struct CAllocator;

/// # Safety
///
/// This allocator delegates to external C callbacks; the caller must ensure
/// those callbacks are correctly implemented.
unsafe impl GlobalAlloc for CAllocator {
    /// 分配内存，对齐大于 `2 * sizeof(usize)` 时使用对齐分配。
    ///
    /// Allocate memory, using aligned allocation when alignment exceeds `2 * sizeof(usize)`.
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if layout.align() > 2 * core::mem::size_of::<usize>() {
            unsafe { shitty_callback_aligned_alloc(layout.size(), layout.align()) }
        } else {
            unsafe { shitty_callback_alloc(layout.size()) }
        }
    }

    /// 释放之前分配的内存。
    ///
    /// Free previously allocated memory.
    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        unsafe { shitty_callback_free(ptr) }
    }

    /// 重新分配内存，必要时扩展或收缩。
    ///
    /// Reallocate memory, growing or shrinking as needed.
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        if layout.align() > 2 * core::mem::size_of::<usize>() {
            unsafe { shitty_callback_aligned_realloc(ptr, new_size, layout.align()) }
        } else {
            unsafe { shitty_callback_realloc(ptr, new_size) }
        }
    }
}

#[global_allocator]
#[cfg(not(feature = "shutup-rust-analyzer"))]
static ALLOC: CAllocator = CAllocator;
