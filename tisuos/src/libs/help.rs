use crate::interrupt::environment::Environment;


extern "C" {
    fn switch_user_process(env : *mut u8) -> usize;
    fn switch_kernel_process(env : *mut u8) -> usize;
}

pub fn switch_kernel(env : &mut Environment) {
    unsafe {
        switch_kernel_process(env as *mut Environment as *mut u8);
    }
}

pub fn switch_user(env : &mut Environment) {
    unsafe {
        switch_user_process(env as *mut Environment as *mut u8);
    }
}
