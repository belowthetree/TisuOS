

extern "C" {
    pub fn switch_user_process(env : *mut u8) -> usize;
    pub fn switch_kernel_process(env : *mut u8) -> usize;
}

