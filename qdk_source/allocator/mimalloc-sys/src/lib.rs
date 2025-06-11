// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use core::ffi::c_void;
pub static MI_ALIGNMENT_MAX: usize = 1024 * 1024; // 1 MiB

// Define core functions from mimalloc needed for the allocator
extern "C" {
    /// Allocate size bytes aligned by alignment.
    /// size: the number of bytes to allocate
    /// alignment: the minimal alignment of the allocated memory. Must be less than `MI_ALIGNMENT_MAX`
    /// returns: a pointer to the allocated memory, or null if out of memory. The returned pointer is aligned by alignment
    pub fn mi_malloc_aligned(size: usize, alignment: usize) -> *mut c_void;
    pub fn mi_zalloc_aligned(size: usize, alignment: usize) -> *mut c_void;

    /// Free previously allocated memory.
    /// The pointer p must have been allocated before (or be nullptr).
    /// p: the pointer to the memory to free or nullptr
    pub fn mi_free(p: *mut c_void);
    pub fn mi_realloc_aligned(p: *mut c_void, newsize: usize, alignment: usize) -> *mut c_void;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memory_can_be_allocated_and_freed() {
        let ptr = unsafe { mi_malloc_aligned(8, 8) }.cast::<u8>();
        assert!(!ptr.cast::<c_void>().is_null());
        unsafe { mi_free(ptr.cast::<c_void>()) };
    }

    #[test]
    fn memory_can_be_allocated_zeroed_and_freed() {
        let ptr = unsafe { mi_zalloc_aligned(8, 8) }.cast::<u8>();
        assert!(!ptr.cast::<c_void>().is_null());
        unsafe { mi_free(ptr.cast::<c_void>()) };
    }

    #[test]
    fn memory_can_be_reallocated_and_freed() {
        let ptr = unsafe { mi_malloc_aligned(8, 8) }.cast::<u8>();
        assert!(!ptr.cast::<c_void>().is_null());
        let realloc_ptr = unsafe { mi_realloc_aligned(ptr.cast::<c_void>(), 8, 8) }.cast::<u8>();
        assert!(!realloc_ptr.cast::<c_void>().is_null());
        unsafe { mi_free(ptr.cast::<c_void>()) };
    }
}
