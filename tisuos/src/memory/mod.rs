//! # 内存管理
//! 内存分为物理内存、虚拟内存管理
//! 目前内存分配以页面为基础，然后形成内存池
//! 程序的内存申请交由堆内存进行管理
//!
//! 2021年1月25日 zg

use self::{
	config::{HEAP_START, KERNEL_PAGE_NUM, MEMORY_END, PAGE_SIZE},
};
use tisu_memory::{MemoryOp, PageManager, Heap};
use core::alloc::{GlobalAlloc, Layout};

pub mod block;
pub mod heap_memory;
pub mod config;
pub mod map;
mod program_memory;
mod stack_memory;

pub use program_memory::*;
pub use stack_memory::*;

type MemoryManager = tisu_memory::MemoryManager<PageManager, Heap<PageManager>>;

static mut USER_HEAP_START : usize = 0;
static mut MANAGER : Option<MemoryManager> = None;

pub fn init(){
	unsafe {
		MANAGER = Some(MemoryManager::new(
			HEAP_START, KERNEL_PAGE_NUM, PAGE_SIZE, MEMORY_END
		));
		USER_HEAP_START = HEAP_START + KERNEL_PAGE_NUM * PAGE_SIZE;
		println!("st {:x}, user {:x}, ed {:x}", HEAP_START, USER_HEAP_START, MEMORY_END);
	}
	// test();
}

#[inline]
pub fn get_manager()->&'static mut impl MemoryOp {
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