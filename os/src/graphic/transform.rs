#![allow(dead_code)]
//! # 变换
//! 固定变换接口，预计可以提供 3D 的变换
//! 2020年12月29日 zg


pub trait ElemTranform {
    fn set_position(&mut self, x : usize, y : usize);
    fn translate(&mut self, x : isize, y : isize);
}



