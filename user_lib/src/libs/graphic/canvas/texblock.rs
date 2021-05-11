//! # 自由排布画布
//! 支持更加灵活的绘制：图片、文字、纯色
//! 
//! 2021年2月5日 zg


pub struct TexBlock {
    pub board : Block<Pixel>,
    pub width : usize,
    pub height : usize,
    pub padding : usize,
}

impl TexBlock {
    pub fn new(width : usize, height : usize, padding : usize)->Self {
        assert!(width > 2 * padding);
        Self {
            width,
            height,
            padding,
            board : Block::<Pixel>::new(width * height),
        }
    }

    pub fn fill_font(&self, x : usize, y : usize, c : char, height : usize, width : usize,
            foreground : Pixel, background : Pixel) {
        let c = c as usize;
        let font = FONT_ASCII[c % 256];
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
                self.board.set(xx + h, v, 1);
                col += 1;
            }
            row += 1;
        }
    }
    pub fn fill(&self, color : Pixel) {
        self.board.set(0, color, self.board.size);
    }
}

impl GridBlock for TexBlock {
    fn transfer_area(&self, width : usize, height : usize)->TexBlock {
        let rt = TexBlock::new(width, height, 0);
        for y in 0..min(rt.height, self.height) {
            rt.board.copy_from(y * rt.width, &self.board, y * self.width, min(self.width, width));
        }
        rt
    }
}

use core::cmp::min;
use tisu_driver::Pixel;

use crate::{libs::{graphic::font::{FONT_ASCII, FONT_HEIGHT, FONT_WIDTH}, memory_block::Block}};

use super::require::GridBlock;

