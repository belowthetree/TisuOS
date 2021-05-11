use core::alloc::{GlobalAlloc, Layout};

use crate::libs::syscall::{free, malloc};

struct OSGlobalAlloc;
unsafe impl GlobalAlloc for OSGlobalAlloc {
    unsafe fn alloc(&self, layout : Layout) -> *mut u8{
        malloc(layout.size()) as *mut u8
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        free(ptr as usize)
    }
}

#[global_allocator]
static GA: OSGlobalAlloc = OSGlobalAlloc{};

#[alloc_error_handler]
pub fn alloc_error(layout : Layout) -> !{
    panic!("Fail to alloc {} bytes with {} bytes alignment", layout.size(), layout.align());
}