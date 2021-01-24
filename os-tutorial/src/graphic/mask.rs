//! # Mask
//! 遮罩，用来显示特定区域
//! 2020年12月30日 zg

/// 圆形遮罩
pub trait Circle{
    /// 将非圆形区域标记为透明
    fn mask(&mut self, radius : usize);
    fn mask_inverse(&mut self, radius : usize);
}


