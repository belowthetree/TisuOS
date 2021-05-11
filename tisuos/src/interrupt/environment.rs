#[allow(dead_code)]
pub enum Register{
    RA = 1,
    SP = 2,
    A0 = 10,
    A1 = 11,
    A2 = 12,
    A3 = 13,
    A4 = 14,
    A5 = 15,
}

impl Register {
    pub fn val(self)->usize{
        self as usize
    }
}

/// 保存需要恢复的环境
#[allow(dead_code)]
#[derive(Copy, Clone, Debug)]
pub struct Environment{
    pub regs    :     [usize;32], // 0 ~ 255
    fregs       :     [usize;32], // 256 ~ 511
    pub satp    :     usize,      // 512
    pub epc     :     usize,      // 520
    pub hartid  :     usize,      // 528
}

#[allow(dead_code)]
impl Environment {
    pub const fn new()->Self{
        Environment{
            regs : [0;32],
            fregs :[0;32],
            satp:  0,
            epc:   0,
            hartid: 0,
        }
    }

    pub fn a0(&self)->usize {
        self.regs[Register::A0.val()]
    }

    pub fn a1(&self)->usize {
        self.regs[Register::A1.val()]
    }

    pub fn a2(&self)->usize {
        self.regs[Register::A2.val()]
    }

    pub fn a3(&self)->usize {
        self.regs[Register::A3.val()]
    }

    pub fn a4(&self)->usize {
        self.regs[Register::A4.val()]
    }

    pub fn a5(&self)->usize {
        self.regs[Register::A5.val()]
    }
}