//! # 内存页管理
//! 将内存分页进行管理，方便后期页表映射
//! 
//! 2021年1月25日 zg

#[derive(Clone, Copy)]
pub struct PageManager {
	kernel_page : *mut Page,
	kernel_page_num : usize,
	user_page : *mut Page,
	user_page_num : usize,
	user_start : usize,
	total_num : usize,
	memory_end : usize,
	page_size : usize,
}

impl PageManager {
	fn clear(&self, addr : *mut u8, num : usize) {
		unsafe {
			let len = num * self.page_size;
			addr.write_bytes(0, len);
		}
	}

	fn init_page(&mut self) {
		let rev_num = (self.total_num + self.page_size - 1) / self.page_size;
		unsafe {
			let ptr = self.kernel_page;
			for i in 0..rev_num {
				(*ptr.add(i)).take();
			}
			for i in rev_num..self.total_num {
				(*ptr.add(i)).free();
			}
		}
		println!("page total {}, rev {}", self.total_num, rev_num);
	}
}

impl PageOp for PageManager {
    fn new(kmem_start : usize, umem_start : usize,
			total_mem : usize, page_size : usize)->Self {
		let kmem_start = (kmem_start + page_size - 1) / page_size * page_size;
		let umem_start = (umem_start + page_size - 1) / page_size * page_size;
		let total_num = (total_mem - kmem_start) / page_size;
		let kernel_page_num = (umem_start - kmem_start) / page_size;
		let mut rt = Self {
		    kernel_page: kmem_start as *mut Page,
		    kernel_page_num: kernel_page_num,
		    user_page: (kmem_start + kernel_page_num) as *mut Page,
		    user_page_num: (total_num - kernel_page_num) / page_size,
			user_start : umem_start,
		    total_num: total_num,
			memory_end : total_num * page_size,
		    page_size: page_size,
		};
		rt.init_page();
		rt
    }

    fn alloc_kernel_page(&mut self, num : usize)->Option<*mut u8> {
		assert!(num > 0);
		unsafe {
			let ptr = self.kernel_page;
			let mut cnt = 0;
			for i in 0..self.kernel_page_num {
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
					let addr = ((i + 1 - cnt) * self.page_size +
						self.kernel_page as usize) as *mut u8;
					self.clear(addr, num);
					return Some(addr);
				}
			}
			None
		}
    }

    fn alloc_user_page(&mut self, num : usize)->Option<*mut u8> {
		assert!(num > 0);
		unsafe {
			let ptr = self.user_page;
			let mut cnt = 0;
			for i in 0..self.user_page_num {
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
					let addr = ((i + 1 - cnt) * self.page_size +
						self.user_page as usize) as *mut u8;
					self.clear(addr, num);
					return Some(addr);
				}
			}
			None
		}
    }

    fn free_page(&mut self, addr : *mut u8) {
		let ptr;
		let mut idx;
		if addr as usize >= self.kernel_page as usize && (addr as usize) < self.user_start {
			idx = (addr as usize - self.kernel_page as usize) / self.page_size;
			ptr = self.kernel_page;
		}
		else if addr as usize >= self.user_start && (addr as usize) < self.memory_end {
			idx = (addr as usize - self.user_start) / self.page_size;
			ptr = self.user_page;
		}
		else {
			panic!("page out of range: {:x}", addr as usize);
		}
		unsafe {
			while !(*ptr.add(idx)).is_end() {
				(*ptr.add(idx)).free();
				idx += 1;
			}
			(*ptr.add(idx)).free();
		}
    }

    fn print(&self) {
		unsafe {
			let mut cnt = 0;
			let ptr = self.kernel_page;
			for i in 0..self.kernel_page_num {
				if !(*ptr.add(i)).is_free() {
					cnt += 1;
				}
				else {
					break;
				}
			}
			println!("kernel use {} pages", cnt);
		}
    }
}

#[derive(Copy, Clone)]
pub struct Page{
	pub flag : u8
}

impl Page {
	pub fn take(&mut self){
		self.flag = PageBit::Taken.val();
	}
	pub fn end(&mut self){
		self.flag |= PageBit::End.val();
	}
	pub fn is_free(&self)->bool{
		self.flag == 0
	}
	pub fn free(&mut self) {
		self.flag = 0;
	}
	pub fn is_end(&self)->bool {
		self.flag & PageBit::End.val() != 0
	}
}

#[derive(Copy, Clone)]
pub enum PageBit{
	Taken = 1 << 0,
	End = 1 << 1,
}

impl PageBit {
	pub const fn val(self) -> u8{
		self as u8
	}
}

pub fn test(){
	print();
	for i in 0..10 {
		let addr = alloc_kernel_page(i + 1).unwrap();
		println!("alloc addr {:x}", addr as usize);
		free_page(addr);
	}
	for i in 0..10 {
		let addr = alloc_kernel_page(i + 1).unwrap();
		println!("alloc addr {:x}", addr as usize);
	}
	print();
}

// #[allow(dead_code)]
// pub fn print(){
// 	unsafe {
// 		let mut cnt = 0;
// 		let mut kernel_cnt = 0;
// 		let ptr = PAGE_START_ADDR;
// 		for i in 0..TOTAL_PAGE {
// 			if (*ptr.add(i)).is_taken() {
// 				cnt += 1;
// 				if (ptr.add(i) as usize) >= KERNEL_PAGE_START_ADDR as usize {
// 					kernel_cnt += 1;
// 				}
// 			}
// 		}
// 		println!("Total page {}, used {}", TOTAL_PAGE, cnt);
// 		println!("Kernel page {}, used {}", KERNEL_PAGE_NUM, kernel_cnt);
// 	}
// }

// use crate::uart;
use super::{alloc_kernel_page, free_page, mem_manager::PageOp, print};
use crate::uart;
