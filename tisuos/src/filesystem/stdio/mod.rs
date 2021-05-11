//! # 标准输入输出缓冲
//! 采用循环队列实现
//!
//! 2021年4月29日 zg

use alloc::collections::BTreeMap;
use tisu_sync::ContentMutex;

mod stdin;
mod stdout;

pub use stdout::Stdout;
pub use stdin::Stdin;

static mut STDIN : ContentMutex<Stdin> = ContentMutex::new(Stdin::new(), true);
static mut STDOUT : ContentMutex<Stdout> = ContentMutex::new(Stdout::new(), true);

pub static mut TASK_IN : 
    Option<ContentMutex<BTreeMap<usize, Stdin>>> = None;
pub static mut TASK_OUT : 
    Option<ContentMutex<BTreeMap<usize, Stdin>>> = None;

pub fn init() {
    unsafe {
        TASK_IN = Some(ContentMutex::new(BTreeMap::new(), true));
        TASK_OUT = Some(ContentMutex::new(BTreeMap::new(), true));
    }
}

pub fn push_task_in(task_id : usize, c : char) {
    unsafe {
        if let Some(tt) = &mut TASK_IN {
            let mut t = tt.lock();
            if let Some(stdin) = t.get_mut(&task_id) {
                stdin.push(c);
            }
            else {
                let mut stdin = Stdin::new();
                stdin.push(c);
                t.insert(task_id, stdin);
            }
        }
    }
}

pub fn push_task_out(task_id : usize, c : char) {
    unsafe {
        if let Some(tt) = &mut TASK_OUT {
            let mut t = tt.lock();
            if let Some(stdout) = t.get_mut(&task_id) {
                stdout.push(c);
            }
            else {
                let mut stdout = Stdin::new();
                stdout.push(c);
                t.insert(task_id, stdout);
            }
        }
    }
}

pub fn pop_task_in(task_id : usize)->Option<char> {
    unsafe {
        if let Some(tt) = &mut TASK_IN {
            let mut t = tt.lock();
            if let Some(stdin) = t.get_mut(&task_id) {
                return stdin.pop();
            }
        }
        None
    }
}

pub fn pop_task_out(task_id : usize)->Option<char> {
    unsafe {
        if let Some(tt) = &mut TASK_OUT {
            let mut t = tt.lock();
            if let Some(stdout) = t.get_mut(&task_id) {
                return stdout.pop();
            }
        }
        None
    }
}

pub fn push_input(c : char) {
    unsafe {
        STDIN.lock().push(c);
    }
}

pub fn pop_input()->Option<char> {
    unsafe {
        STDIN.lock().pop()
    }
}

pub fn push_output(c : char) {
    unsafe {
        STDOUT.lock().push(c);
    }
}

pub fn pop_output()->Option<char> {
    unsafe {
        STDOUT.lock().pop()
    }
}