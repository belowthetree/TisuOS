//! BMP
//! 提供 BMP 格式图片操控接口
//! 2020年12月31日 zg

#[repr(packed)]
#[allow(dead_code)]
pub struct BMP{
    btype : u16,
    size : u32,
    reserved : u32,
    data_offset : u32,
    info_size : u32,
    width : u32,
    height : i32,
    plane : u16,
    bitcnt : u16,
    compression_type : CompressType,
    image_size : u32,
    pixel_x : i32,
    pixel_y : i32,
    cnt_used : u32, // 彩色表中颜色索引数，为 0 使用所有
    important_cnt : u32, // 有重要影响的颜色索引数目
}

impl BMP {
    pub fn is_bmp(&self)->bool{
        // println!("{}BM", &from_u16(self.btype)[..]);
        &from_u16(self.btype)[..] == "BM"
    }
}

pub fn generate_image(file : File)->Option<Image>{
    let content;
    if let Some(ctx) = file.read(0, file.size){
        content = ctx;
    }
    else{
        return None;
    }
    let addr = content.addr as *const BMP;
    unsafe {
        let bmp = &*addr;
        if bmp.is_bmp() && !bmp.compression_type.is_compressed(){
            let width = bmp.width as usize;
            let height = bmp.height.abs() as usize;
            let is_down = bmp.height > 0;
            let mut rt = Image::new(width, height, Format::RGB);
            let mut data_offset = bmp.data_offset as usize;
            let mut idx = 0;
            let ptr = content.addr;
            let skip = (4 - (width * bmp.bitcnt as usize) % 4) % 4;
            // println!("file size {}KB, skip {} per line, size of color {}",
            //     file.size / 1024, skip, size_of::<BMPColor>());
            for _ in 0..height{
                for _ in 0..width{
                    let color = *(ptr.add(data_offset) as *mut BMPColor);
                    rt.set(idx, Pixel{r:color.r,g:color.g,b:color.b,a:color.a});
                    idx += 1;
                    data_offset += 4;
                }
                data_offset += skip;
            }
            if is_down {
                rt.updown();
            }
            println!("gen img over");
            Some(rt)
        }
        else{
            println!("not bmp {}", from_u16((addr as *mut u16).read_volatile()));
            None
        }
    }
}
#[repr(packed)]
#[derive(Clone, Copy)]
struct BMPColor{
    b:u8,
    g:u8,
    r:u8,
    a:u8
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CompressType{
    RGB = 0,
}
impl CompressType{
    pub fn is_compressed(&self)->bool{
        *self != CompressType::RGB
    }
}


use core::mem::size_of;

use crate::{filesystem::file::{File}, libs::str::from_u16, uart, virtio::{gpu_device::Pixel}};

use super::image::{Format, Image};

