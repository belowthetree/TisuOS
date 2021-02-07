//! # 网格画布
//! 
//! 2021年1月31日 zg


pub struct Grid {
    pub grid_width : usize,
    pub grid_height : usize,
    pub height : usize,
    pub width : usize,
    pub line_num : usize,
    pub buffer : Block<Pixel>,
    pub display_line : usize,
    pub block_num : usize,
    pub colorblock : ColorBlock,
    pub refresh : bool,
}

#[allow(dead_code)]
impl Grid {
    /// ### 初始化成纯色
    pub fn solid_color(x : usize, y : usize, width : usize, height : usize, w : usize, h : usize, color : Pixel)->Self {
        assert!(width / w > 0);
        let buffer = Block::new(height * width * 2);
        buffer.set(0, color, buffer.size);
        Self {
            grid_width : w,
            grid_height : h,
            height : height * 2,
            width : width,
            line_num : width / w,
            buffer : buffer,
            display_line : 0,
            block_num : width / w * (height * 2 / h),
            colorblock : ColorBlock::solid_color(x, y, width, height, color),
            refresh : false,
        }
    }
    pub fn image(x : usize, y : usize, image : &Image, w : usize, h : usize)->Self {
        let width = image.width;
        let height = image.height;
        assert!(width / w > 0);
        Self {
            grid_width : w,
            grid_height : h,
            height : height,
            width : width,
            line_num : width / w,
            buffer : Block::new(height * width),
            display_line : 0,
            block_num : width / w * (height / h),
            colorblock : ColorBlock::image(x, y, image),
            refresh : false,
        }
    }

}

impl Grid {
    /// ### 移动
    pub fn translate(&mut self, vec : Vector) {
        self.colorblock.translate(vec);
    }
    /// ### 检测点位在哪个格子里
    pub fn detect(&self, point : Position)->Option<usize> {
        let x1 = self.colorblock.x;
        let y1 = self.colorblock.y;
        let x2 = x1 + self.line_num * self.grid_width;
        let y2 = y1 + self.height;
        
        if point.x < x1 || point.y < y1 || point.x > x2 || point.y > y2 {
            return None;
        }

        let x = point.x - x1;
        let y = point.y - y1;
        let idx = y / self.grid_height * self.line_num + x / self.grid_width;

        if idx < self.block_num {
            Some(idx)
        }
        else {
            None
        }
    }
}

/// ## 绘制
impl Grid {
    /// ### 直接覆盖到显示缓冲区
    pub fn draw_override(&mut self) {
        if self.refresh {
            self.transfer();
        }
        self.colorblock.draw_override();
    }
    pub fn draw_blend(&mut self) {
        if self.refresh {
            self.transfer();
        }
        self.colorblock.draw_blend();
    }
    /// ### 在 idx 对应的各自里填充颜色
    pub fn fill_color(&mut self, idx : usize, color : Pixel) {
        if idx >= self.block_num {
            self.expand();
        }
        let x = idx % self.line_num * self.grid_width;
        let y = idx / self.line_num * self.grid_height;
        for i in y..(y + self.grid_height) {
            self.buffer.set(x + i * self.width, color, self.grid_width);
        }
        self.refresh = true;
    }
    /// ### 在 idx 对应的各自里填充字符
    pub fn fill_font(&mut self, idx : usize, c : char, foreground : Pixel, background : Pixel) {
        if idx >= self.block_num {
            self.expand();
        }
        let c = c as usize;
        let font = FONT_ASCII[c % 256];
        let x = idx % self.line_num * self.grid_width;
        let y = idx / self.line_num * self.grid_height;
        let height = self.grid_height;
        let width = self.grid_width;
        let mut row = 0;

        for yy in y..(y + height) {
            if yy >= self.height as usize {
                break;
            }
            let h = yy * self.width as usize;
            let r = row * FONT_HEIGHT / height;
            let mut col = 0;
            for xx in x..(x + width) {
                if xx >= self.width {
                    break;
                }
                let co = col * FONT_WIDTH / width;
                let v;
                if font[r] & (1 << (7 - co)) != 0 {
                    v = foreground;
                }
                else {
                    v = background;
                }
                self.buffer.set(xx + h, v, 1);
                col += 1;
            }
            row += 1;
        }
        self.refresh = true;
    }
    /// ### 从某个画布块中获取一块内容进行填充
    pub fn from_block<T:GridBlock>(&mut self, idx : usize, block : &T, alpha : bool) {
        if idx >= self.block_num {
            self.expand();
        }
        let block = block.transfer_area(self.grid_width, self.grid_height);
        let width = min(self.grid_width, block.width);
        let height = min(self.grid_height, block.height);
        let x = idx % self.line_num * self.grid_width;
        let y = idx / self.line_num * self.grid_height;
        
        if alpha {
            let t = 1.0 / 255.0;
            let mut idx = 0;
            for y in y..(y + height) {
                let mut ii = idx * block.width;
                let y = y * self.width;
                for x in x..(x + width) {
                    let id = x as usize + y;
                    let color1 = block.board.get(ii).unwrap();
                    let color2 = self.buffer.get(id).unwrap();
                    let rate =  color1.a as f32 * t;
                    let rate2 = 1.0 - rate;
                    let color = Pixel{
                        r : (color1.r as f32 * rate) as u8 + (color2.r as f32 * rate2) as u8,
                        g : (color1.g as f32 * rate) as u8 + (color2.g as f32 * rate2) as u8,
                        b : (color1.b as f32 * rate) as u8 + (color2.b as f32 * rate2) as u8,
                        a : (color1.a as f32 * rate) as u8 + (color2.a as f32 * rate2) as u8,
                    };
                    self.buffer.set(id, color, 1);
                    ii += 1;
                }
                idx += 1;
            }
        }
        else {
            let mut yy = 0;
            for y in y..(y + height) {
                self.buffer.copy_from(x + y * self.width, &block.board, yy * block.width, width);
                yy += 1;
            }
        }
        self.refresh = true;
    }
}

impl Grid {
    fn transfer(&mut self) {
        self.colorblock.buffer.copy_from(0, &self.buffer,
            self.display_line * self.width, self.width * self.height);
        self.refresh = false;
    }
    fn expand(&mut self) {
        let buffer = Block::<Pixel>::new(self.width * (self.height + self.colorblock.height));
        buffer.copy_from(0, &self.buffer, 0, self.width * self.height);
        self.buffer = buffer;
        self.height = self.height + self.colorblock.height;
        self.block_num = self.width / self.grid_width * (self.height / self.grid_height);
    }
    pub fn scroll(&mut self, offset : isize) {
        let mut line = self.display_line as isize + offset;
        if line < 0 {
            line = 0;
        }
        else if line as usize > self.height - self.colorblock.height {
            line = (self.height - self.colorblock.height) as isize;
        }
        self.display_line = line as usize;
        self.refresh = true;
    }
    pub fn fill(&mut self, color : Pixel) {
        self.display_line = 0;
        self.buffer.set(0, color, self.buffer.size);
        self.refresh = true;
    }
}



use core::cmp::min;

use crate::{filesystem::image::image::Image, graphic::colorblock::ColorBlock, libs::{font::{FONT_ASCII, FONT_HEIGHT, FONT_WIDTH}, graphic::Pixel, shape::{Position, Vector}}, memory::block::Block};

use super::{require::GridBlock, texblock::{self, TexBlock}};
use crate::uart;

