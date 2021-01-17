#![allow(dead_code)]

pub fn get_hartid() -> usize {
    unsafe {
        let id;
        asm!("csrr $0, mhartid" : "=r"(id));
        id
    }
}

pub fn read_sstatus() -> usize {
    unsafe {
        let rt;
        asm!("csrr $0, sstatus" : "=r"(rt));
        rt
    }
}

pub fn write_sstatus(val : usize) {
    unsafe {
        asm!("csrw sstatus, $0"::"r"(val));
    }
}

pub fn write_satp(val : usize) {
    unsafe {
        asm!("csrw satp, $0"::"r"(val));
    }
}

pub fn read_satp() -> usize {
    unsafe {
        let rt;
        asm!("csrr $0, satp" : "=r"(rt));
        rt
    }
}

pub fn write_stvec(val : usize) {
    unsafe {
        asm!("csrw stvec, $0"::"r"(val));
    }
}

pub fn read_stvec() -> usize {
    unsafe {
        let rt;
        asm!("csrr $0, stvec" : "=r"(rt));
        rt
    }
}

pub fn read_mscratch() -> usize {
    unsafe {
        let rt;
        asm!("csrr $0, mscratch" : "=r"(rt));
        rt
    }
}

pub fn write_scratch(val : usize) {
    unsafe {
		asm!("csrw	mscratch, $0" ::"r"(val));
	}
}

pub fn satp_fence_asid(asid: usize) {
	unsafe {
		asm!("sfence.vma zero, $0" :: "r"(asid));
	}
}

