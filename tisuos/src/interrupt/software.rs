//! # 软件中断功能
//!
//! 2021年5月31日 zg


pub fn activate(hartid : usize) {
    unsafe {
        let ptr = 0x2000000 as *mut u32;
        ptr.add(hartid).write_volatile(1);
    }
}

pub fn inactivate(hartid : usize) {
    unsafe {
        let ptr = 0x2000000 as *mut u32;
        ptr.add(hartid).write_volatile(0);
    }
}