//! # Page
//! 以页面为基本单位管理内存
//! 2020年11月 zg
use core::{ptr::null_mut, mem::size_of};

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
}

use crate::uart;
#[repr(u8)]
#[derive(Copy, Clone)]
pub enum PageBit{
	Empty = 1 << 0,
	Taken = 1 << 1,
	End = 1 << 2,
	User = 1 << 3,
	Kernel = 1 <<4,
	Write = 1 << 5,
	Read = 1 << 6,
	Reserved = 1 << 7,
}
impl PageBit {
	pub const fn val(self) -> u8{
		self as u8
	}
}
#[derive(Copy, Clone)]
pub struct Page{
	pub flag : u8
}
#[allow(dead_code)]
impl Page {
	pub const fn new(flag : PageBit) -> Self{
		Page{flag : flag.val()}
	}
	pub fn free(&mut self){
		if !self.is_reserved(){
			self.flag = PageBit::Empty.val();
		}
	}
	pub fn take(&mut self){
		self.flag = PageBit::Taken.val();
	}
	pub fn reserved(&mut self){
		self.flag = PageBit::Taken.val() | PageBit::Reserved.val();
	}
	pub fn end(&mut self){
		self.flag |= PageBit::End.val();
	}
	pub fn is_free(&self)->bool{
		self.flag & PageBit::Empty.val() == PageBit::Empty.val()
	}
	pub fn is_reserved(&self)->bool{
		self.flag & PageBit::Reserved.val() != 0
	}
	pub fn is_taken(&self)->bool{
		self.flag & PageBit::Taken.val() == PageBit::Taken.val()
	}
	pub fn is_end(&self)->bool{
		self.flag & PageBit::End.val() == PageBit::End.val()
	}
	pub fn set_flag(&mut self, flag : u8){
		self.flag |= flag;
	}
	pub fn set_user_read_write(&mut self){
		self.flag |= PageBit::User.val() | PageBit::Read.val() | PageBit::Write.val(); 
	}
	pub fn set_kernel_read_write(&mut self){
		self.flag |= PageBit::Kernel.val() | PageBit::Read.val() | PageBit::Write.val(); 
	}
	pub fn is_kernel(&self) -> bool {
		self.flag & PageBit::Kernel.val() != 0
	}
	pub fn is_user(&self) -> bool {
		self.flag & PageBit::User.val() != 0
	}
}

pub const  PAGE_SIZE : usize = 4096;
pub const KERNEL_PAGE_NUM : usize = 20480;
pub static mut PAGES : *mut Page = null_mut();
pub static mut TOTAL_PAGE : usize = 0;
pub static mut NUM_RESERVE : usize = 0;
pub static mut KERNEL_HEAP_START : usize = 0;
pub static mut KERNEL_PAGE_STRUCT_START : *mut Page = null_mut();
pub static mut USER_HEAP_START : usize = 0;
pub static mut USER_PAGE_NUM : usize = 0;
pub static mut USER_PAGE_STRUCT_START : *mut Page = null_mut();

/// ## 初始化
/// 页表结构体放在 KERNEL_HEAP 起始处，连同 MMIO 区域在内作为保留区域
/// 可分配页表在这之后
/// 此外将页表分为内核与用户两部分，分别用于内核与用户的内存分配
pub fn init(){
	unsafe {
		PAGES = HEAP_START as *mut Page;
		TOTAL_PAGE = MEMORY_END / PAGE_SIZE;
		let ptr = PAGES;
		for i in 0..TOTAL_PAGE {
			(*ptr.add(i)).free();
		}
		
		let num_reserved = (TOTAL_PAGE * size_of::<Page>() + HEAP_START + PAGE_SIZE - 1) / PAGE_SIZE;
		//println!("total page num: {}, revered num:{}", TOTAL_PAGE, num_reserved);
		for i in 0..num_reserved {
			(*ptr.add(i)).reserved();
		}
		NUM_RESERVE = num_reserved;
		KERNEL_HEAP_START = num_reserved * PAGE_SIZE;
		KERNEL_PAGE_STRUCT_START = ptr.add(num_reserved);
		USER_HEAP_START = KERNEL_HEAP_START + KERNEL_PAGE_NUM * PAGE_SIZE;
		USER_PAGE_NUM = TOTAL_PAGE - num_reserved - KERNEL_PAGE_NUM;
		USER_PAGE_STRUCT_START = ptr.add(num_reserved + KERNEL_PAGE_NUM);
		//println!("kernel page start 0x{:x}, user page start 0x{:x}", KERNEL_HEAP_START, USER_HEAP_START);
	}
}

/// ## 申请给定数量的页表，标记为用户
/// 不负责清零
pub fn alloc_user_page(num : usize) -> *mut u8 {
	assert!(num > 0);
	unsafe {
		let ptr = USER_PAGE_STRUCT_START;
		let mut cnt = 0;
		for i in 0..USER_PAGE_NUM {
			if (*ptr.add(i)).is_free() {
				cnt += 1;
			}
			else {
				cnt = 0;
			}
			if cnt >= num {
				for idx in i + 1 - cnt..=i {
					(*ptr.add(idx)).take();
					(*ptr.add(idx)).set_user_read_write();
				}
				(*ptr.add(i)).end();
				let addr = ((i + 1 - cnt) * PAGE_SIZE + USER_HEAP_START) as *mut u8;
				return addr;
			}
		}
		null_mut()
	}
}
/// ## 申请内核用的页
/// 分配前清零
pub fn alloc_kernel_page(num : usize) -> *mut u8{
	assert!(num > 0);
	unsafe {
		let ptr = KERNEL_PAGE_STRUCT_START;
		let mut cnt = 0;
		for i in 0..KERNEL_PAGE_NUM {
			if (*ptr.add(i)).is_free() {
				cnt += 1;
			}
			else {
				cnt = 0;
			}
			if cnt >= num {
				for idx in i + 1 - cnt..=i {
					(*ptr.add(idx)).take();
					(*ptr.add(idx)).set_kernel_read_write();
				}
				(*ptr.add(i)).end();
				let addr = ((i + 1 - cnt) * PAGE_SIZE + KERNEL_HEAP_START) as *mut u8;
				clear_page(addr, num);
				//println!("give {:x}", addr as usize);
				return addr;
			}
		}
		null_mut()
	}
}
/// 释放给定地址对应的页表
pub fn free_page(addr : *mut u8) {
	assert!(!addr.is_null());
	unsafe {
		let n = (addr as usize) / PAGE_SIZE;
		if n <= NUM_RESERVE {
			return;
		}
		let ptr = KERNEL_PAGE_STRUCT_START as *mut Page;
		let mut idx = n - NUM_RESERVE;
		while !(*ptr.add(idx)).is_end() {
			(*ptr.add(idx)).free();
			idx += 1;
		}
		(*ptr.add(idx)).free();
	}
}

/// ## 打印页表分配信息
/// 用于调试
pub fn print_page_alloc(){
	unsafe {
		let mut free_cnt = 0;
		let mut alloc_cnt = 0;
		let mut unknown_cnt = 0;
		let ptr = KERNEL_HEAP_START as *mut Page;
		for i in 0..KERNEL_PAGE_NUM + USER_PAGE_NUM {
			if (*ptr.add(i)).is_free() {
				free_cnt += 1;
			}
			else if (*ptr.add(i)).is_taken() {
				alloc_cnt += 1;
			}
			else {
				unknown_cnt += 1;
			}
		}
		//println!("Page info start at 0x{:08x}, end at 0x{:08x}", PAGES as usize, alloc_cnt * PAGE_SIZE);
		println!("Page alloc num:{}, free num:{}, unknown num:{}", 
			alloc_cnt, free_cnt, unknown_cnt);
	}
}
/// ## 清理页面
/// 在分配前进行清零操作，速度较慢
pub fn clear_page(addr : *mut u8, num : usize){
	let len = num * PAGE_SIZE / 8;
	let ptr = addr as *mut u64;
	unsafe {
		ptr.write_bytes(0, len);
	}
}