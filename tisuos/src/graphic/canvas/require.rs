//! # 某些扩展用接口要求
//! 
//! 2021年2月5日 zg


use super::texblock::TexBlock;

pub trait GridBlock {
    /// ### 用于传输对应大小的区域给 Grid 绘制
    /// 目前不进行大小放缩
    fn transfer_area(&self, width : usize, height : usize)->TexBlock;
}