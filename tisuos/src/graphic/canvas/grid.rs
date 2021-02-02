//! # 网格画布
//! 
//! 2021年1月31日 zg


pub struct Grid {
    grid_width : usize,
    grid_height : usize,
    height : usize,
    width : usize,
    line_num : usize,
    buffer : Block<Pixel>,
    display_line : usize,
    block_num : usize,
    colorblock : ColorBlock,
    refresh : bool,
}

impl Grid {
    pub fn new(x : usize, y : usize, width : usize, height : usize, w : usize, h : usize)->Self {
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
            colorblock : ColorBlock::solid_color(x, y, width, height, Pixel::black()),
            refresh : false,
        }
    }
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
    pub fn draw_override(&mut self) {
        if self.refresh {
            self.transfer();
        }
        self.colorblock.draw_override();
    }
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
    pub fn scroll(&mut self, offset : isize) {
        let mut line = self.display_line as isize + offset;
        if line < 0 {
            line = 0;
        }
        else if line as usize > self.height - self.colorblock.height {
            line = (self.height - self.colorblock.height) as isize;
        }
        self.display_line = line as usize;
    }
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
}


use core::cmp::min;

use crate::{graphic::colorblock::ColorBlock, libs::{font::{FONT_ASCII, FONT_HEIGHT, FONT_WIDTH}, graphic::Pixel}, memory::block::Block};
use crate::uart;

