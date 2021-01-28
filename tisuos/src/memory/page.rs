//! # 内存页管理
//! 将内存分页进行管理，方便后期页表映射
//! 
//! 2021年1月25日 zg

pub const PHYSIC_BASE : usize = 0x1000_0000;
pub const PAGE_SIZE : usize = 4096;
pub const KERNEL_PAGE_NUM : usize = 51200;

#[derive(Copy, Clone)]
pub struct Page{
	pub flag : u8
}

impl Page {
	pub const fn new(flag : PageBit) -> Self{
		Page{flag : flag.val()}
	}
	pub fn free(&mut self){
		self.flag = 0;
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
		self.flag == 0
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
}

#[derive(Copy, Clone)]
pub enum PageBit{
	Taken = 1 << 0,
	End = 1 << 1,
	Reserved = 1 << 6,
}

impl PageBit {
	pub const fn val(self) -> u8{
		self as u8
	}
}

pub static mut PAGE_START_ADDR : *mut Page = null_mut();
pub static mut TOTAL_PAGE : usize = 0;
pub static mut NUM_RESERVE : usize = 0;
pub static mut USER_PAGE_START_ADDR : *mut Page = null_mut();
pub static mut KERNEL_PAGE_START_ADDR : *mut Page = null_mut();
pub static mut USER_PAGE_NUM : usize = 0;
pub static mut KERNEL_HEAP_START : usize = 0;
pub static mut USER_HEAP_START : usize = 0;


pub fn init(){
	unsafe {
		PAGE_START_ADDR = HEAP_START as *mut Page;
		TOTAL_PAGE = MEMORY_END / PAGE_SIZE;
		NUM_RESERVE = (TOTAL_PAGE * size_of::<Page>() + HEAP_START + PAGE_SIZE - 1) / PAGE_SIZE;
		let ptr = PAGE_START_ADDR;
		for i in 0..NUM_RESERVE {
			(*ptr.add(i)).reserved();
		}
		for i in NUM_RESERVE..TOTAL_PAGE {
			(*ptr.add(i)).free();
		}
		KERNEL_PAGE_START_ADDR = PAGE_START_ADDR.add(NUM_RESERVE);
		USER_PAGE_START_ADDR = KERNEL_PAGE_START_ADDR.add(KERNEL_PAGE_NUM);
		USER_PAGE_NUM = TOTAL_PAGE - KERNEL_PAGE_NUM;
		KERNEL_HEAP_START = NUM_RESERVE * PAGE_SIZE;
		USER_HEAP_START = KERNEL_HEAP_START + KERNEL_PAGE_NUM * PAGE_SIZE;
		println!("Kernel heap at {:x}", KERNEL_HEAP_START);
	}
}

/// ## 从用户内存页区申请给定数量的页表
/// 不负责清零
pub fn alloc_user_page(num : usize) -> *mut u8 {
	assert!(num > 0);
	unsafe {
		let ptr = USER_PAGE_START_ADDR;
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
		let ptr = KERNEL_PAGE_START_ADDR;
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

/// ## 释放给定地址对应的页表
pub fn free_page(addr : *mut u8) {
	assert!(!addr.is_null());
	unsafe {
		let n = (addr as usize) / PAGE_SIZE;
		if n <= NUM_RESERVE {
			return;
		}
		let ptr = PAGE_START_ADDR as *mut Page;
		let mut idx = n - NUM_RESERVE;
		while !(*ptr.add(idx)).is_end() {
			(*ptr.add(idx)).free();
			idx += 1;
		}
		(*ptr.add(idx)).free();
	}
}

fn clear_page(addr : *mut u8, num : usize){
	unsafe {
		addr.write_bytes(0, num * PAGE_SIZE);
	}
}

pub fn test(){
	for i in 0..10 {
		let addr = alloc_kernel_page(i + 1);
		println!("alloc addr {:x}", addr as usize);
	}
	print();
}

#[allow(dead_code)]
pub fn print(){
	unsafe {
		let mut cnt = 0;
		let mut kernel_cnt = 0;
		let ptr = PAGE_START_ADDR;
		for i in 0..TOTAL_PAGE {
			if (*ptr.add(i)).is_taken() {
				cnt += 1;
				if (ptr.add(i) as usize) >= KERNEL_PAGE_START_ADDR as usize {
					kernel_cnt += 1;
				}
			}
		}
		println!("Total page {}, used {}", TOTAL_PAGE, cnt);
		println!("Kernel page {}, used {}", KERNEL_PAGE_NUM, kernel_cnt);
	}
}

use core::{mem::size_of, ptr::null_mut};
use crate::uart;
use super::{HEAP_START, MEMORY_END};
