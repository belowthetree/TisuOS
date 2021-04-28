
global_asm!(include_str!("../func.S"));

extern  "C" {
    fn env_call(num:usize, a0 : usize)->usize;
}

const TEST : usize = 1;

pub fn syscall_test() {
    syscall(TEST, 0);
}

pub fn syscall_test2() {
    syscall(2, 0);
}

fn syscall(num:usize, a0 : usize) {
    unsafe {
        env_call(num, a0);
    }
}