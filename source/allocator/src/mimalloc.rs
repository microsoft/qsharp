// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use core::alloc::{GlobalAlloc, Layout};
use core::ffi::c_void;

use mimalloc_sys::{mi_free, mi_malloc_aligned, mi_realloc_aligned, mi_zalloc_aligned};

pub struct Mimalloc;

unsafe impl GlobalAlloc for Mimalloc {
    #[inline]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        debug_assert!(layout.align() < mimalloc_sys::MI_ALIGNMENT_MAX);
        mi_malloc_aligned(layout.size(), layout.align()).cast::<u8>()
    }

    #[inline]
    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        mi_free(ptr.cast::<c_void>());
    }

    #[inline]
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        debug_assert!(layout.align() < mimalloc_sys::MI_ALIGNMENT_MAX);
        mi_zalloc_aligned(layout.size(), layout.align()).cast::<u8>()
    }

    #[inline]
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        debug_assert!(layout.align() < mimalloc_sys::MI_ALIGNMENT_MAX);
        mi_realloc_aligned(ptr.cast::<c_void>(), new_size, layout.align()).cast::<u8>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn memory_can_be_allocated_and_freed() -> Result<(), Box<dyn Error>> {
        let layout = Layout::from_size_align(8, 8)?;
        let alloc = Mimalloc;

        unsafe {
            let ptr = alloc.alloc(layout);
            assert!(!ptr.cast::<c_void>().is_null());
            alloc.dealloc(ptr, layout);
        }
        Ok(())
    }

    #[test]
    fn memory_can_be_alloc_zeroed_and_freed() -> Result<(), Box<dyn Error>> {
        let layout = Layout::from_size_align(8, 8)?;
        let alloc = Mimalloc;

        unsafe {
            let ptr = alloc.alloc_zeroed(layout);
            assert!(!ptr.cast::<c_void>().is_null());
            alloc.dealloc(ptr, layout);
        }
        Ok(())
    }

    #[test]
    fn large_chunks_of_memory_can_be_allocated_and_freed() -> Result<(), Box<dyn Error>> {
        let layout = Layout::from_size_align(2 * 1024 * 1024 * 1024, 8)?;
        let alloc = Mimalloc;

        unsafe {
            let ptr = alloc.alloc(layout);
            assert!(!ptr.cast::<c_void>().is_null());
            alloc.dealloc(ptr, layout);
        }
        Ok(())
    }
}
