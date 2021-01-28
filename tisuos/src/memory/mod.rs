//! # 内存管理
//! 采用对齐内存方式分配堆栈
//! 内存分页进行管理，提供详细的内存操作支持
//! 
//! 2021年1月25日 zg

pub mod allocator;
pub mod page;
pub mod block;
pub mod page_table;
pub mod user_allocator;
mod bitmap;

pub fn init(){
    page::init();
}

#[allow(dead_code)]
pub fn test(){
    allocator::test();
    page::test();
}

#[allow(dead_code)]
extern "C" {
    pub static TEXT_START: usize;
	pub static TEXT_END: usize;
	pub static DATA_START: usize;
	pub static DATA_END: usize;
	pub static RODATA_START: usize;
	pub static RODATA_END: usize;
	pub static BSS_START: usize;
	pub static BSS_END: usize;
	pub static KERNEL_STACK_START: usize;
	pub static KERNEL_STACK_END: usize;
	pub static HEAP_START: usize;
    pub static HEAP_SIZE: usize;
    pub static MEMORY_START: usize;
	pub static MEMORY_END: usize;
	pub static TRAP_STACK_END : usize;
}