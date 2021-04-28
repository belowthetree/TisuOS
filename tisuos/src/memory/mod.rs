//! # 内存管理
//! 采用对齐内存方式分配堆栈
//! 内存分页进行管理，提供详细的内存操作支持
//! 
//! 2021年1月25日 zg

use self::{
	config::{HEAP_START, KERNEL_PAGE_NUM, MEMORY_END, PAGE_SIZE},
};
use tisu_memory::{MemoryOp, PageManager, Heap};
use core::alloc::{GlobalAlloc, Layout};

pub mod block;
pub mod user_allocator;
pub mod config;
pub mod map;

type MemoryManager = tisu_memory::MemoryManager<PageManager, Heap<PageManager>>;

static mut USER_HEAP_START : usize = 0;
static mut MANAGER : Option<MemoryManager> = None;

pub fn init(){
	unsafe {
		MANAGER = Some(MemoryManager::new(
			HEAP_START, KERNEL_PAGE_NUM, PAGE_SIZE, MEMORY_END
		));
		USER_HEAP_START = HEAP_START + KERNEL_PAGE_NUM * PAGE_SIZE;
	}
	// test();
}

#[inline]
pub fn get_manager()->&'static mut MemoryManager {
	unsafe {
		let mut rt = None;
		if let Some(mgr) = &mut MANAGER {
			rt = Some(mgr);
		}
		rt.unwrap()
	}
}

#[allow(dead_code)]
pub fn test(){
	let mgr = get_manager();
	for _ in 0..10 {
		println!("addr {:x}", mgr.user_page(1).unwrap() as usize);
	}
	for _ in 0..100 {
		let addr = mgr.user_page(1).unwrap();
		println!("addr {:x}", addr as usize);
		mgr.free_page(addr);
	}
}

#[allow(dead_code)]
pub fn print() {
	unsafe {
		if let Some(mgr) = &mut MANAGER {
			mgr.print();
		}
	}
}

pub fn alloc(size : usize, is_kernel : bool)->Option<*mut u8> {
	unsafe {
		if let Some(mgr) = &mut MANAGER {
			mgr.alloc_memory(size, is_kernel)
		}
		else{None}
	}
}

pub fn free(addr : *mut u8) {
	unsafe {
		if let Some(mgr) = &mut MANAGER {
			mgr.free_memory(addr);
		}
		else{panic!("Error {}", 0)}
	}
}

struct OSGlobalAlloc;
unsafe impl GlobalAlloc for OSGlobalAlloc {
    unsafe fn alloc(&self, layout : Layout) -> *mut u8{
        get_manager().alloc_memory(layout.size(), true).unwrap()
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        get_manager().free_memory(ptr);
    }
}

#[global_allocator]
static GA: OSGlobalAlloc = OSGlobalAlloc{};

#[alloc_error_handler]
pub fn alloc_error(layout : Layout) -> !{
    panic!("Fail to alloc {} bytes with {} bytes alignment", layout.size(), layout.align());
}