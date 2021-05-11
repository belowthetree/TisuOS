//! # 堆内存池
//! 将小块堆内存切分，加速分配
//!
//! 2021年4月28日 zg
#![allow(dead_code)]

use tisu_memory::MemoryOp;
use crate::memory::{block::Block, config::PAGE_SIZE, get_manager, map::SATP};

/// 16 KB 内使用内存池
pub const MAX_BLOCK_SIZE : usize = 1024 * 16;
/// 内存池倍率
pub const POOL_RATE : usize = 16;

pub struct HeapPool {
    physic_addr : usize,
    pub virtual_base : usize,
    pub virtual_top : usize,
    page_num : usize,
    pub block_size : usize,
    pub total_size : usize,
    record : &'static mut [bool],
    use_num : usize,
    block_num : usize,
    inner : Block<bool>,
    is_kernel : bool,
}

fn align(n : usize)->usize {
    let mut rt = 2;
    while rt < n {
        rt = rt << 1;
    }
    rt
}

impl HeapPool {
    pub fn new(virtual_addr : usize, block_size : usize, is_kernel : bool)->Self {
        let block_size = align(block_size);
        let size;
        if block_size <= MAX_BLOCK_SIZE { size = block_size * POOL_RATE; }
        else { size = block_size; }
        let page_num = (size + PAGE_SIZE - 1) / PAGE_SIZE;
        let physic_addr;
        if is_kernel {
            physic_addr = get_manager().kernel_page(page_num).unwrap();
        }
        else {
            physic_addr = get_manager().user_page(page_num).unwrap();
        }
        let block_num = PAGE_SIZE * page_num / block_size;
        let inner = Block::new(block_num);
        let total_size = PAGE_SIZE * page_num;
        Self {
            physic_addr : physic_addr as usize,
            virtual_base : virtual_addr,
            virtual_top : total_size + virtual_addr,
            page_num,
            total_size : total_size,
            block_size,
            record : inner.array(0, block_num),
            use_num : 0,
            block_num,
            inner,
            is_kernel
        }
    }

    /// 给出虚拟地址
    pub fn alloc(&mut self)->Option<(usize, usize)> {
        if self.use_num >= self.block_num {
            None
        }
        else {
            for i in 0..self.block_num {
                if !self.record[i] {
                    self.use_num += 1;
                    self.record[i] = true;
                    let t = i * self.block_size;
                    return Some((t + self.virtual_base, t + self.physic_addr));
                }
            }
            panic!("heap pool alloc err total {} use {} size {}",
                self.use_num, self.block_num, self.block_size);
        }
    }

    pub fn free(&mut self, va : usize) {
        let offset = va - self.virtual_base;
        let idx = offset / self.block_size;
        assert!(self.record[idx]);
        self.record[idx] = false;
        self.use_num -= 1;
    }

    pub fn map(&self, satp : &SATP) {
        for i in 0..self.page_num {
            let t = i * PAGE_SIZE;
            let pa = self.physic_addr + t;
            let va = self.virtual_base + t;
            // println!("satp {:x} heap map vst {:x} pst {:x} {}", satp.flag, va, pa, self.is_kernel);
            satp.map_data(va, pa, self.is_kernel);
        }
    }

    pub fn full(&self)->bool {
        self.use_num == self.block_num
    }

    pub fn contain(&self, va : usize)->bool {
        self.virtual_base <= va && self.virtual_top > va
    }

    pub fn virt_to_phy(&self, va : usize)->usize {
        self.physic_addr + va - self.virtual_base
    }
}


impl Drop for HeapPool {
    fn drop(&mut self) {
        // println!("heap pool drop {:x}", self.physic_addr);
        get_manager().free_page(self.physic_addr as *mut u8);
    }
}
