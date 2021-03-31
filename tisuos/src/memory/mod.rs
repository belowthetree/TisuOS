//! # 内存管理
//! 采用对齐内存方式分配堆栈
//! 内存分页进行管理，提供详细的内存操作支持
//! 
//! 2021年1月25日 zg

use self::{allocator::Allocator, config::{HEAP_START, KERNEL_PAGE_NUM, MEMORY_END, PAGE_SIZE}, mem_manager::MemoryManager, page::PageManager};

mod allocator;
mod page;
mod bitmap;
mod mem_manager;
pub mod block;
pub mod page_table;
pub mod user_allocator;
pub mod config;

pub static mut MEM_MANAGER : Option<MemoryManager<PageManager, Allocator<PageManager>>> = None;
static mut USER_HEAP_START : usize = 0;

pub fn init(){
	unsafe {
		USER_HEAP_START = HEAP_START + KERNEL_PAGE_NUM * PAGE_SIZE;
		MEM_MANAGER = Some(MemoryManager::new());
	}
	// test();
}

#[allow(dead_code)]
pub fn test(){
    page::test();
    allocator::test();
}

pub fn print() {
	unsafe {
		if let Some(mgr) = &mut MEM_MANAGER {
			mgr.print();
		}
	}
}

pub fn alloc_kernel_page(num : usize)->Option<*mut u8> {
	unsafe {
		if let Some(mgr) = &mut MEM_MANAGER {
			mgr.kernel_page(num)
		}
		else{None}
	}
}

pub fn alloc_user_page(num : usize)->Option<*mut u8> {
	unsafe {
		if let Some(mgr) = &mut MEM_MANAGER {
			mgr.user_page(num)
		}
		else{None}
	}
}

pub fn free_page(addr : *mut u8) {
	unsafe {
		if let Some(mgr) = &mut MEM_MANAGER {
			mgr.free_page(addr)
		}
		else{panic!("Error {}", 0)}
	}
}

pub fn alloc(size : usize, is_kernel : bool)->Option<*mut u8> {
	unsafe {
		if let Some(mgr) = &mut MEM_MANAGER {
			mgr.alloc_memory(size, is_kernel)
		}
		else{None}
	}
}

pub fn free(addr : *mut u8) {
	unsafe {
		if let Some(mgr) = &mut MEM_MANAGER {
			if addr as usize >= HEAP_START && (addr as usize) < USER_HEAP_START {
				mgr.free_kernel_memory(addr);
			}
			else if addr as usize >= USER_HEAP_START && (addr as usize) < MEMORY_END {
				mgr.free_user_memory(addr);
			}
			else {
				panic!("free memory fail addr: {:x}", addr as usize);
			}
		}
		else{panic!("Error {}", 0)}
	}
}
