#![allow(dead_code)]

pub fn hold_on() {
    unsafe {
        asm!(
            "wfi"
        );
    }
}

pub fn add_epc(n : usize) {
    unsafe {
        asm!(
            "
            add t0, zero, {n}
            csrw mepc, t0
            ",
            n = in(reg) n
        );
    }
}

pub fn write_satp(satp : usize) {
    unsafe {
        asm!(
            "
            mv t0, {satp}
            csrw satp, t0
            ",
            satp = in(reg) satp
        )
    }
}
use core::arch::asm;