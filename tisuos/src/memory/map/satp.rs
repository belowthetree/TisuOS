//! ## SATP 管理器
//! 管理所有映射行为
//!
//! 2021年4月27日 zg

use core::cmp::{max, min};

use tisu_memory::MemoryOp;
use crate::{
    filesystem::elf::ElfManager,
    memory::{config::{KERNEL_STACK_START, MEMORY_END, MEMORY_START, PAGE_SIZE},
    get_manager}
};
use super::page_table::PageTable;

const ASID : usize = 0;
const MODE : usize = 8;
extern "C" {
    fn thread_exit();
    fn process_exit();
    fn waiting();
}

#[derive(Debug, Clone, Copy)]
pub struct SATP{
    pub flag : usize,
}

/// 信息相关方法
impl SATP {
    pub fn new() -> Self{
        SATP{
            flag : 0,
        }
    }

    pub fn from(satp : usize)->Self {
        Self {
            flag : satp,
        }
    }

    pub fn set(&mut self, ppn:usize) {
        self.flag = ((ppn >> 12) & 0xfff_ffff_ffff)
        | ((ASID & 0xffff) << 44) | ((MODE & 0xf) << 60)
    }

    pub fn get_ppn_addr(&self)->usize{
        (self.flag & 0xfff_ffff_ffff) << 12
    }

    pub fn val(&self) -> usize{
        self.flag as usize
    }

    pub fn get_page_table(&self) ->&mut PageTable {
        unsafe {
            if self.is_map() {
                &mut *(self.get_ppn_addr() as *mut PageTable)
            }
            else {
                let pt = PageTable::new();
                let t = self as *const Self as *mut Self;
                println!("before set {:x}", self.flag);
                (*t).set(pt as *mut PageTable as usize);
                println!("before set {:x}", self.flag);
                pt
            }
        }
    }

    pub fn is_map(&self)->bool{
        self.get_ppn_addr() != 0
    }
}

/// 操作方法
impl SATP {
    pub fn map_data(&self, va:usize, pa:usize, is_kernel:bool) {
        let pt = self.get_page_table();
        if is_kernel {
            pt.map_kernel_data(va, pa);
        }
        else {
            pt.map_user_data(va, pa);
        }
    }

    pub fn map_code(&self, va:usize, pa:usize, is_kernel:bool) {
        let pt = self.get_page_table();
        if is_kernel {
            pt.map_kernel_code(va, pa);
        }
        else {
            pt.map_user_code(va, pa);
        }
    }

    pub fn map_all(&self, va:usize, pa:usize, is_kernel:bool) {
        let pt = self.get_page_table();
        if is_kernel {
            pt.map_kernel(va, pa);
        }
        else {
            pt.map_user(va, pa);
        }
    }

    pub fn free_page_table(&self){
        if self.is_map(){
            let pt = self.get_page_table();
            pt.free();
        }
    }

    pub fn map_kernel_area(&self, is_kernel : bool){
        unsafe {
            let pt = self.get_page_table();
            if is_kernel{
                let st = MEMORY_START;
                let ed = KERNEL_STACK_START;
                for addr in (st..ed).step_by(PAGE_SIZE){
                    pt.map_kernel(addr, addr);
                }
                let med = MEMORY_END;
                for addr in (ed..med).step_by(PAGE_SIZE){
                    pt.map_kernel_data(addr,addr);
                }
                for i in 0..10{
                    let addr =0x1000_0000 + i * PAGE_SIZE;
                    pt.map_kernel(addr,addr);
                }
                let addr = 0x200_0000;
                for i in 0..16 {
                    let addr = addr + i * PAGE_SIZE;
                    pt.map_kernel_data(addr,addr);
                }
            }
            else {
                let st = min(process_exit as usize, thread_exit as usize);
                let st = min(st, waiting as usize);
                let ed = max(process_exit as usize, thread_exit as usize);
                let ed = max(ed, waiting as usize);
                for addr in (st..=ed).step_by(PAGE_SIZE){
                    pt.map_user_code(addr, addr);
                }
            }
        }
    }

    /// 此方法自动为程序段分配页表
    pub fn map_elf(&self, elf : &mut ElfManager, is_kernel:bool) {
        elf.reset();
        let mgr =get_manager();
        while let Some(ph) = elf.next_ph() {
            if !ph.is_loadable() {
                continue;
            }
            let va = ph.va();
            let offset = ph.va() % PAGE_SIZE;
            let num = (ph.size() + offset + PAGE_SIZE - 1) / PAGE_SIZE;
            let pa = if is_kernel{mgr.kernel_page(num)}else{mgr.user_page(num)};
            let pa = pa.unwrap();
            unsafe {pa.add(offset).copy_from(elf.get_addr(ph.offset()), ph.size())}
            let pa = pa as usize;
            for i in 0..num {
                let va = va + i * PAGE_SIZE;
                let pa = pa + i * PAGE_SIZE;
                self.map_all(va, pa, is_kernel);
            }
        }
    }
}
