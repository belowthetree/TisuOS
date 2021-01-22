//! # Style
//! 控件样式的支持
//! 2020年12月30日 zg

pub struct Style {
    pub color_style : ColorStyle,
    pub color : Pixel,
    pub texture : Option<Image>,
    pub element : Element,
}

#[derive(Clone, Copy, PartialEq)]
pub enum ColorStyle{
    Texture,
    SolidColor,
}

const DEFAULT_COLOR : Pixel = Pixel{r:122,g:122,b:255,a:255};

impl Style {
    pub fn new(color_style : ColorStyle, x:u32, y:u32, width : u32, height : u32)->Self{
        let mut elem = Element::new(x, y, width, height);
        elem.fill(DEFAULT_COLOR);
        Self{
            color_style : color_style,
            color : DEFAULT_COLOR,
            texture : None,
            element : elem,
        }
    }
    pub fn new_default()->Self{
        let mut elem = Element::new(0, 0, 50, 20);
        elem.fill(DEFAULT_COLOR);
        Self{
            color_style : ColorStyle::SolidColor,
            color : DEFAULT_COLOR,
            texture : None,
            element : elem,
        }
    }
    pub fn resize(&mut self, width : u32, height : u32){
        self.element.resize(width, height);
        match self.color_style {
            ColorStyle::SolidColor => {
                self.element.fill(self.color);
            }
            ColorStyle::Texture => {
                if let Some(tex) = &self.texture{
                    let ptr = self.element.content.addr as *mut Pixel;
                    for y in 0..height as usize{
                        let yy = y * tex.height / height as usize;
                        let tt = yy * tex.width;
                        let t = y * width as usize;
                        for x in 0..width as usize{
                            unsafe {
                                let xx = x * tex.width / width as usize;
                                (*ptr.add(x + t)) = *(tex.data.addr as *mut Pixel).add(xx + tt);
                            }
                        }
                    }
                }
            }
        }
    }
    pub fn set_color(&mut self, color : Pixel){
        self.element.fill(color);
        self.color = color;
        self.color_style = ColorStyle::SolidColor;
    }
    pub fn draw(&self){
        self.element.draw();
    }
    pub fn set_texture(&mut self, image : Image){
        self.color_style = ColorStyle::Texture;
        self.texture = Some(image);
    }
}

impl Transform for Style{
    fn set_position(&mut self, x : u32, y : u32) {
        self.element.x = x;
        self.element.y = y;
    }

    fn maximum(&mut self) {
    }

    fn minimum(&mut self) {
    }

    fn detect(&mut self, point : Position)->bool {
        let x = point.x;
        let y = point.y;
        self.element.x <= x && self.element.y <= y && self.element.x + self.element.width >= x && self.element.y + self.element.height >= y
    }

    fn translate(&mut self, x : i32, y : i32) {
        let mut x = self.element.x as i32 + x;
        let mut y = self.element.y as i32 + y;
        if x < 0{
            x = 0;
        }
        if y < 0{
            y = 0;
        }
        self.element.x = x as u32;
        self.element.y = y as u32;
    }
}

use crate::{desktop::{desktop::Position, desktop_trait::Transform}, filesystem::image::image::Image};
use crate::{graphic::element::{Element, Draw}, virtio::gpu_device::Pixel};

