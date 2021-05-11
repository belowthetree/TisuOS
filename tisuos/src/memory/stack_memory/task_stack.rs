//! # 任务用栈管理
//!
//! 2021年4月29日 zg
#![allow(dead_code)]

use alloc::prelude::v1::*;

pub struct StackArea {
    vst : usize,
    ved : usize,
    pst : usize,
    ped : usize,
}

pub struct TaskStack {
    stack_top : usize,
    stack_bottom : usize,
    area : Vec<StackArea>,
}

impl TaskStack {

}
