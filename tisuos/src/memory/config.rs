//! # 内存常量记录
//! 
//! 2021年3月27日 zg

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

pub const PAGE_SIZE : usize = 4096;
pub const KERNEL_PAGE_NUM : usize = 25600;