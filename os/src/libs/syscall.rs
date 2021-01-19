//! # 系统调用封装
//! 2021年1月6日 zg

extern "C" {
    pub fn make_syscall(num : usize, arg1 : usize, arg2 : usize, arg3 : usize, arg4 : usize) -> usize;
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

pub fn exec(func : usize, satp : usize){
    unsafe {
        make_syscall(4, func, satp, 0, 0);
    }
}

pub fn list_thread(){
    unsafe {
        make_syscall(5, 0, 0, 0, 0);
    }
}