#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
#[repr(packed)]
pub struct Ip {
    version_and_headlen : u8,
    server : u8,
    total_len : u16,
    sign : u16,
    sign_and_offset : u16,
    src_ip : u32,
    target_ip : u32,
}

impl Ip {
    pub fn new()->Self {
        Self {
            version_and_headlen : (5 << 4) | 4,
            server : 0,
            total_len : 20,
            sign : 0x1ce3,
            sign_and_offset : 0,
            src_ip : (192 << 24) | (198 << 16) | 121,
            target_ip : (192 << 24) | (198 << 16) | 121,
        }
    }
}