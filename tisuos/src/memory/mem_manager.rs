//! # 内存管理器
//! 管理页面、堆内存分配算法
//! 
//! 2021年3月27日 zg

use crate::sync::Mutex;

use super::{MEMORY_END, config::{HEAP_START, KERNEL_PAGE_NUM, PAGE_SIZE}};

pub struct MemoryManager<T1 : PageOp, T2 : MemoryOp<T1>> {
    pub page : T1,
    pub memory : T2,
    memory_lock : Mutex,
    page_lock : Mutex,
}

impl<T1 : PageOp + Clone + Copy, T2 : MemoryOp<T1>> MemoryManager<T1, T2> {
    pub fn new()->Self {
        let user_heap = unsafe{HEAP_START + KERNEL_PAGE_NUM * PAGE_SIZE};
        let page = T1::new(unsafe{HEAP_START},
            user_heap, unsafe{MEMORY_END}, PAGE_SIZE);
        Self {
            page: page,
            memory: T2::new(&mut page.clone()),
            memory_lock : Mutex::new(),
            page_lock : Mutex::new(),
        }
    }

    pub fn kernel_page(&mut self, num : usize)->Option<*mut u8> {
        self.page_lock.lock();
        let rt = self.page.alloc_kernel_page(num);
        self.page_lock.unlock();
        rt
    }

    pub fn user_page(&mut self, num : usize)->Option<*mut u8> {
        self.page_lock.lock();
        let rt = self.page.alloc_user_page(num);
        self.page_lock.unlock();
        rt
    }

    pub fn free_page(&mut self, addr : *mut u8) {
        self.page_lock.lock();
        self.page.free_page(addr);
        self.page_lock.unlock();
    }

    pub fn alloc_memory(&mut self, size : usize, is_kernel : bool)->Option<*mut u8> {
        self.memory_lock.lock();
        let rt;
        if is_kernel {
            rt = self.memory.alloc_kernel_memory(size)
        }
        else {
            rt = self.memory.alloc_user_memory(size)
        }
        self.memory_lock.unlock();
        rt
    }

    pub fn free_kernel_memory(&mut self, addr : *mut u8) {
        self.memory_lock.lock();
        self.memory.free_kernel_memory(addr);
        self.memory_lock.unlock();
    }

    pub fn free_user_memory(&mut self, addr : *mut u8) {
        self.memory_lock.lock();
        self.memory.free_user_memory(addr);
        self.memory_lock.unlock();
    }

    pub fn print(&mut self) {
        self.page_lock.lock();
        self.page.print();
        self.page_lock.unlock();
        self.memory_lock.lock();
        self.memory.print();
        self.memory_lock.unlock();
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