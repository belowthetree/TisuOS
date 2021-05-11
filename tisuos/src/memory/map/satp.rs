//! ## SATP 管理器
//! 负责所有映射操作
//!
//! 2021年4月27日 zg

use super::page_table::PageTable;

const ASID : usize = 0;
const MODE : usize = 8;

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
                (*t).set(pt as *mut PageTable as usize);
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
            if pa == 0x84471000 {
                println!("map data {:x} {:x}", va, pa);
            }
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
}
