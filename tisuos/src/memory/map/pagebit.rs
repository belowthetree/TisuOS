#[allow(dead_code)]
#[repr(u64)]
pub enum PageBit{
    Valid   = 1 << 0,
    Read    = 1 << 1,
    Write   = 1 << 2,
    Excute  = 1 << 3,
    User    = 1 << 4,
    Global  = 1 << 5,
    Access  = 1 << 6,
    Dirty   = 1 << 7,
}

impl PageBit {
    pub fn val(self) -> u64 {
        self as u64
    }
}
