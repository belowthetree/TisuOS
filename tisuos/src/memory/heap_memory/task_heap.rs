//! # 用户堆管理器
//! 用于处理用户的堆内存申请
//! 在虚拟地址维护一个向上增长的堆，同时管理对应的物理页面。堆内存以内存池方式管理
//!
//! 2021年4月29日 zg


use alloc::vec::Vec;
use crate::{memory::map::SATP, task::process::MAX_HEAP_SIZE};

use super::heap_pool::HeapPool;

/// ## 链接同一个进程的所有堆内存
#[allow(dead_code)]
pub struct TaskHeap{
    /// 记录堆开始的虚拟地址
    virtual_heap_start : usize,
    virtual_heap_top : usize,
    memory_area : Vec<HeapPool>,
    is_kernel : bool,
}


/// 以虚拟地址交互，不涉及物理地址
impl TaskHeap {
    /// 根据大小新建一个，结构体本身存放在内核堆内存里，用户所需在用户内存中申请
    pub fn new(virtual_heap_start : usize, is_kernel : bool)->Self {
        Self {
            virtual_heap_start,
            virtual_heap_top : virtual_heap_start,
            memory_area: Vec::new(),
            is_kernel,
        }
    }

    pub fn alloc(&mut self, size : usize, satp : &SATP)->(usize, usize) {
        if let Some(pool) = self.memory_area.iter_mut().find(|pool| {
            pool.block_size >= size && !pool.full()
        }) {
            pool.alloc().expect("task heap pool alloc err")
        }
        else {
            self.expand(size, satp);
            let pool = self.memory_area.iter_mut().find(|pool| {
                pool.block_size >= size && !pool.full()
            }).expect("heap pool find err");
            pool.alloc().expect("task heap pool alloc err after expand")
        }
    }

    pub fn free(&mut self, va : usize) {
        let pool = self.memory_area.iter_mut().find(|pool| {
            pool.contain(va)
        });
        if pool.is_none() {
            panic!("free Error va {:x}", va);
        }
        pool.unwrap().free(va);
    }

    pub fn virt_to_phy(&self, va : usize)->usize {
        let pool = self.memory_area.iter().find(|pool| {
            pool.contain(va)
        }).unwrap();
        pool.virt_to_phy(va)
    }

    fn expand(&mut self, size : usize, satp : &SATP) {
        let pool = HeapPool::new(
            self.virtual_heap_top, size, self.is_kernel);
        pool.map(satp);
        self.virtual_heap_top += pool.total_size;
        self.memory_area.push(pool);
        self.memory_area.sort_by(|a, b| {
            a.block_size.cmp(&b.block_size)
        });
        assert!(self.virtual_heap_top - self.virtual_heap_start <= MAX_HEAP_SIZE);
    }
}

