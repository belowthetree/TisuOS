//! # 内存管理器
//! 管理页面、堆内存分配算法
//! 
//! 2021年3月27日 zg


use core::alloc::{GlobalAlloc, Layout};
use crate::sync::content::ContentMutex;

use super::{config::MEMORY_END, config::{HEAP_START, KERNEL_PAGE_NUM, PAGE_SIZE}, free, alloc};

pub struct MemoryManager<T1 : PageOp, T2 : MemoryOp<T1>> {
    page : ContentMutex<T1>,
    memory : ContentMutex<T2>,
}

impl<T1 : PageOp + Clone + Copy, T2 : MemoryOp<T1>> MemoryManager<T1, T2> {
    pub fn new()->Self {
        let user_heap = unsafe{HEAP_START + KERNEL_PAGE_NUM * PAGE_SIZE};
        let page = T1::new(unsafe{HEAP_START},
            user_heap, unsafe{MEMORY_END}, PAGE_SIZE);
        Self {
            page: ContentMutex::new(page),
            memory: ContentMutex::new(T2::new(&mut page.clone())),
        }
    }

    pub fn kernel_page(&mut self, num : usize)->Option<*mut u8> {
        let mut page = self.page.lock();
        (*page).alloc_kernel_page(num)
    }

    pub fn user_page(&mut self, num : usize)->Option<*mut u8> {
        let mut page = self.page.lock();
        (*page).alloc_user_page(num)
    }

    pub fn free_page(&mut self, addr : *mut u8) {
        let mut page = self.page.lock();
        (*page).free_page(addr);
    }

    pub fn alloc_memory(&mut self, size : usize, is_kernel : bool)->Option<*mut u8> {
        let mut memory = self.memory.lock();
        if is_kernel {
            (*memory).alloc_kernel_memory(size)
        }
        else {
            (*memory).alloc_user_memory(size)
        }
    }

    pub fn free_kernel_memory(&mut self, addr : *mut u8) {
        let mut memory = self.memory.lock();
        (*memory).free_kernel_memory(addr);
    }

    pub fn free_user_memory(&mut self, addr : *mut u8) {
        let mut memory = self.memory.lock();
        (*memory).free_user_memory(addr);
    }

    pub fn print(&mut self) {
        let page = self.page.lock();
        (*page).print();
        let memory = self.memory.lock();
        (*memory).print();
    }
}


/// ## 页面管理需要提供的对外接口
pub trait PageOp : Clone {
    fn new(kmem_start : usize, umem_start : usize,
        total_mem : usize, page_size : usize)->Self;
    /// ### 申请内核用页面
    fn alloc_kernel_page(&mut self, num : usize)->Option<*mut u8>;
    /// ### 申请用户用页面
    fn alloc_user_page(&mut self, num : usize)->Option<*mut u8>;
    fn free_page(&mut self, addr : *mut u8);
    fn page_size(&self)->usize;
    fn print(&self);
}

pub trait MemoryOp<T:PageOp> {
    fn new(page : &mut T)->Self;
    fn alloc_kernel_memory(&mut self, size : usize)->Option<*mut u8>;
    fn alloc_user_memory(&mut self, size : usize)->Option<*mut u8>;
    fn free_kernel_memory(&mut self, addr : *mut u8);
    fn free_user_memory(&mut self, addr : *mut u8);
    fn print(&self);
}


/// ## 容器内存管理
/// 实现 RUST 容器的内存分配 trait
/// 所有内存在内核部分分配
struct OSGlobalAlloc;
unsafe impl GlobalAlloc for OSGlobalAlloc {
    unsafe fn alloc(&self, layout : Layout) -> *mut u8{
        alloc(layout.size(), true).unwrap()
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        free(ptr);
    }
}

#[global_allocator]
static GA: OSGlobalAlloc = OSGlobalAlloc{};

#[alloc_error_handler]
pub fn alloc_error(layout : Layout) -> !{
    panic!("Fail to alloc {} bytes with {} bytes alignment", layout.size(), layout.align());
}

