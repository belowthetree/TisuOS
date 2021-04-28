//! # 系统调用封装
//! 2021年1月6日 zg

use alloc::prelude::v1::*;

extern "C" {
    fn make_syscall(num : usize, arg1 : usize, arg2 : usize, arg3 : usize, arg4 : usize) -> usize;
}

// #[macro_export]
// macro_rules! fork {
//     () => {
//         unsafe {
//             asm!("ecall"::"a0"(57)::"volatile");
//         }
//     };
// }

pub fn fork()->usize{
    unsafe {
        make_syscall(57, 0, 0, 0, 0)
    }
}

#[allow(dead_code)]
pub fn exec(path : String)->usize {
    unsafe {
        let mut c = Vec::<char>::new();
        for ch in path.as_bytes() {
            c.push(*ch as char);
        }
        let addr = c.as_slice() as *const [char] as *const char as usize;
        make_syscall(4, addr, path.len(), 0, 0)
    }
}

pub fn list_thread(){
    unsafe {
        make_syscall(5, 0, 0, 0, 0);
    }
}
#[allow(dead_code)]
pub fn branch(func : usize)->usize {
    unsafe {
        make_syscall(7, func, 0, 0, 0)
    }
}

