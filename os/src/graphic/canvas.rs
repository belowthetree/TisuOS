//! # 画布
//! 作为 element 单元的补充，提供翻页功能
//! 
//! 2021年1月23日 zg

use super::element::Element;

pub struct Canvas{
    pub element : Element,
    pub page_height : usize,
    pub page_width : usize,
    pub page_num : usize,
    pub display_offset : usize,
}