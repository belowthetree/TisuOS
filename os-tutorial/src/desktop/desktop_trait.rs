//! # 桌面接口
//! 要求桌面元素全体支持，接口与底层 Graphic 紧密结合
//! 2020年12月30日 zg


use super::desktop::Position;
/// ## 变化要求
pub trait Transform {
    fn set_position(&mut self, x : usize, y : usize);
    fn translate(&mut self, x : isize, y : isize);
    /// ## 用于检测桌面元素是否在包含此点
    /// 按钮的状态应该在此函数内完成判断、转变
    fn detect(&mut self, point : Position)->bool;
    fn refresh(&mut self);
}

pub trait Trigger {
    fn click(&mut self);
}