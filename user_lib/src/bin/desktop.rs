#![no_std]
#![no_main]
#![feature(
    alloc_prelude,
)]

extern crate alloc;
extern crate user_lib;
use alloc::prelude::v1::*;
use fs_format::BMP;
use user_lib::libs::{fs, graphic::{ColorBlock, HEIGHT, WIDTH}};


#[no_mangle]
extern "C" fn _start(){
    let data = fs::File::read("1/img/mac.bmp".to_string()).unwrap();
    let mut image = BMP::decode(data.to_array(0, data.size)).unwrap();
    image.resize(WIDTH, HEIGHT);
    let color = ColorBlock::image(0, 0, &image);
    color.draw_override();
}