//! # 程序内存区域
//!
//! 2021年4月29日 zg

use core::cmp::{max, min};

use crate::{filesystem::elf::ElfManager, memory::{config::{DATA_START, HEAP_START, KERNEL_PAGE_NUM, MEMORY_START, PAGE_SIZE, RODATA_END}, get_manager, map::SATP}};
use alloc::prelude::v1::*;
use tisu_memory::MemoryOp;

extern "C" {
    fn thread_exit();
    fn process_exit();
    fn waiting();
}

#[derive(Debug)]
pub struct Area {
    vst : usize,
    ved : usize,
    pst : usize,
    ped : usize,
    atype : AreaType,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AreaType {
    Code,
    Data,
    All,
}

impl Area {
    pub fn new(vst : usize, ved : usize, pst: usize, ped: usize)->Self {
        Self {
            vst,
            ved,
            pst,
            ped,
            atype : AreaType::All
        }
    }

    pub fn kernel_data()->Self {
        let vst = unsafe {DATA_START};
        let ved = unsafe {HEAP_START + KERNEL_PAGE_NUM * PAGE_SIZE};
        let ved = ved / PAGE_SIZE * PAGE_SIZE;
        let vst = vst / PAGE_SIZE * PAGE_SIZE;
        println!("data vst {:x} ved {:x}", vst, ved);
        Self {
            vst,
            ved,
            pst : vst,
            ped : ved,
            atype : AreaType::Data
        }
    }

    pub fn kernel_code()->Self {
        let vst = unsafe {MEMORY_START};
        let ved = unsafe {RODATA_END};
        // println!("code vst {:x} ved {:x}", vst, ved);
        let ved = ved / PAGE_SIZE * PAGE_SIZE;
        let vst = vst / PAGE_SIZE * PAGE_SIZE;
        Self {
            vst,
            ved,
            pst : vst,
            ped : ved,
            atype : AreaType::Code
        }
    }

    pub fn virtio_area()->Self {
        let vst = 0x1000_0000;
        let ved = 0x1000_9000;
        Self {
            vst,
            ved,
            pst : vst,
            ped:ved,
            atype : AreaType::Data
        }
    }

    pub fn timer_area()->Self {
        let vst = 0x200_0000;
        let ved = 0x200_C000;
        Self {
            vst,
            ved,
            pst : vst,
            ped:ved,
            atype : AreaType::Data
        }
    }

    /// 内核中用户程序会用到的函数
    pub fn user_func()->Self {
        let st = min(process_exit as usize, thread_exit as usize);
        let st = min(st, waiting as usize);
        let ed = max(process_exit as usize, thread_exit as usize);
        let ed = max(ed, waiting as usize);
        let ed = ed / PAGE_SIZE * PAGE_SIZE;
        let st = st / PAGE_SIZE * PAGE_SIZE;
        Self {
            vst:st,
            ved:ed,
            pst:st,
            ped:ed,
            atype:AreaType::Code
        }
    }

    pub fn contain(&self, va:usize)->bool {
        self.vst <= va && self.ved > va
    }

    pub fn virt_to_phy(&self, va:usize)->usize {
        self.pst + va - self.vst
    }
}

pub struct ProgramArea {
    entry : usize,
    area : Vec<Area>,
    pub is_kernel : bool
}

impl ProgramArea {
    pub fn new(entry:usize, is_kernel : bool)->Self {
        Self {
            entry,
            area : Vec::new(),
            is_kernel
        }
    }

    pub fn entry(&self)->usize {
        self.entry
    }

    pub fn push_area(&mut self, area : Area) {
        self.area.push(area);
    }

    pub fn push_elf(&mut self, elf : &mut ElfManager) {
        elf.reset();
        let mgr = get_manager();
        while let Some(ph) = elf.next_ph() {
            if !ph.is_loadable() {
                continue;
            }
            let va = ph.va();
            let offset = ph.va() % PAGE_SIZE;
            let num = (ph.size() + offset + PAGE_SIZE - 1) / PAGE_SIZE;
            let pa = if self.is_kernel{mgr.kernel_page(num)}else{mgr.user_page(num)};
            let pa = pa.unwrap();
            unsafe {pa.add(offset).copy_from(elf.get_addr(ph.offset()), ph.size())}
            let pa = pa as usize;
            let vst = va;
            let ved = va + num * PAGE_SIZE;
            let pst = pa;
            let ped = pa + num * PAGE_SIZE;
            self.push_area(Area::new(vst, ved, pst, ped));
        }
        if self.is_kernel {
            self.push_area(Area::kernel_code());
            self.push_area(Area::kernel_data());
            self.push_area(Area::virtio_area());
            self.push_area(Area::timer_area());
        }
        else {
            self.push_area(Area::user_func());
        }
    }

    pub fn map(&self, satp : &SATP) {
        for area in self.area.iter() {
            let mut vst = area.vst;
            let mut pst = area.pst;
            while vst <= area.ved && pst <= area.ped {
                if vst == 0x84471000 {
                    println!("vst {:x}, pst {:x}", vst, pst);
                }
                // if 0x84346e90 <= vst && self.is_kernel && area.atype == AreaType::Data {
                //     println!("satp {:x} vst: {:x}, pst: {:x}", satp.flag, vst, pst);
                // }
                match area.atype {
                    AreaType::Code => satp.map_code(vst, pst, self.is_kernel),
                    AreaType::Data => satp.map_data(vst, pst, self.is_kernel),
                    AreaType::All => satp.map_all(vst, pst, self.is_kernel)
                }
                vst += PAGE_SIZE;
                pst += PAGE_SIZE;
            }
        }
        if self.is_kernel {
            println!("kernel satp {:x}", satp.flag);
        }
    }

    pub fn virt_to_phy(&self, va:usize)->usize {
        // println!("va {:x}", va);
        let area = self.area.iter().find(|area| {
            // println!("vst {:x} ved {:x}", area.vst, area.ved);
            area.contain(va)
        }).unwrap();
        area.virt_to_phy(va)
    }

    pub fn contain(&self, va:usize)->bool {
        self.area.iter().find(|area| {
            area.contain(va)
        }).is_some()
    }
}

impl Drop for ProgramArea {
    fn drop(&mut self) {
        let mgr = get_manager();
        for area in self.area.iter() {
            let addr = area.pst;
            // HEAP_START 以下的地址不是内存分配所得，不需要回收
            if addr < unsafe {HEAP_START} {
                continue;
            }
            mgr.free_page(addr as *mut u8);
        }
    }
}
