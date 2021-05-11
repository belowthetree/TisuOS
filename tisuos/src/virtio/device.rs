//! # Device
//! 虚拟IO设备管理
//! 开机扫描 0x1000_1000 到 0x1000_8000 的内存地址，查找存在的设备并初始化、记录
//! 2020年11月 zg

const VIRTIO_START_ADDR : usize = 0x1000_1000;
const VIRTIO_END_ADDR : usize = 0x1000_8000;
const VIRTIO_STEP_SIZE : usize = 0x1000;
const VIRTIO_VAL : u32 = 0x74726976;

type BlockDevice = tisu_driver::Block;
type GraphicDevice = tisu_driver::GPU;
type InputDevice = tisu_driver::InputDevice;

/// ## 全局设备管理
pub struct Device {
    pub block_device : Vec<Box<dyn BlockDriver>>,
    pub gpu_device : Vec<Box<dyn GraphicDriver>>,
    pub input_device : Vec<Box<dyn Driver>>,
    dtype : [(DeviceType, usize);9],
}

/// 中断通过 pending 通知对应设备，通过 handler 进行处理
impl Device {
    pub fn new()->Self {
        let mut rt = Self {
            block_device : vec![],
            gpu_device : vec![],
            input_device : vec![],
            dtype : [(DeviceType::Unknown, 0);9],
        };
        rt.search_device();
        rt
    }

    fn search_device(&mut self) {
        for i in (VIRTIO_START_ADDR..=VIRTIO_END_ADDR).step_by(VIRTIO_STEP_SIZE){
            let header = i as *mut u32;
            let magic;
            let device_id;
            unsafe {
                magic = header.read_volatile();
                device_id = header.add(2).read_volatile();
            }
            if magic == VIRTIO_VAL {
                let idx = (i - VIRTIO_START_ADDR) / VIRTIO_STEP_SIZE + 1;
                self.add_device(header as *mut VirtHeader, idx, device_id);
            }
        }
    }

    pub fn add_device(&mut self, header : *mut VirtHeader, pin_idx : usize, device_id : u32) {
        let memory = get_manager();
        match DeviceType::from(device_id as usize) {
            DeviceType::Block => {
                self.dtype[pin_idx] = (DeviceType::Block, self.block_device.len());
                let b = Box::new(BlockDevice::new(header,memory));
                self.block_device.push(b);
            }
            DeviceType::Gpu => {
                self.dtype[pin_idx] = (DeviceType::Block, self.gpu_device.len());
                let b = Box::new(GraphicDevice::new(header, WIDTH, HEIGHT, memory));
                self.gpu_device.push(b);
            }
            DeviceType::Input => {
                self.dtype[pin_idx] = (DeviceType::Input, self.input_device.len());
                let dev = InputDevice::new(header, memory);
                self.input_device.push(Box::new(dev));
            }
            _ => {}
        }
    }

    pub fn pending(&mut self, pin_idx : usize) {
        match self.dtype[pin_idx].0 {
            DeviceType::Block => {
                let blk = self.block_device.get_mut(self.dtype[pin_idx].1).unwrap();
                blk.pending().unwrap();
            }
            DeviceType::Gpu => {
                let gpu = self.gpu_device.get_mut(self.dtype[pin_idx].1).unwrap();
                gpu.pending().unwrap();
            }
            DeviceType::Input => {
                let input = self.input_device.get_mut(self.dtype[pin_idx].1).unwrap();
                let e = input.pending();
                if let Ok(InterruptOk::Input(e)) = e {
                    println!("{:?}", e);
                }
            }
            _ => {}
        }
    }

    pub fn handler(&mut self, pin_idx : usize) {
        match self.dtype[pin_idx].0 {
            DeviceType::Block => {
                let blk = self.block_device.get_mut(self.dtype[pin_idx].1).unwrap();
                blk.handler().unwrap();
            }
            DeviceType::Gpu => {
                let gpu = self.gpu_device.get_mut(self.dtype[pin_idx].1).unwrap();
                gpu.handler().unwrap();
            }
            DeviceType::Input => {
                let input = self.input_device.get_mut(self.dtype[pin_idx].1).unwrap();
                while let Ok(InterruptOk::Input(e)) = input.handler() {
                    if let Ok(a) = Decoder::decode(
                        e.etype as usize, e.code as usize, e.value as usize){
                        match a {
                            virtio_input_decoder::DecodeType::Key(key, ktype) => {
                                match ktype {
                                    virtio_input_decoder::KeyType::Press => {
                                        add_key_press(key as usize);
                                        if let Ok(c) = key.to_char() {
                                            push_input(c);
                                        }
                                    }
                                    virtio_input_decoder::KeyType::Release => {
                                        add_key_release(key as usize)
                                    }
                                }
                            }
                            virtio_input_decoder::DecodeType::Mouse(mouse) => {
                                match mouse {
                                    virtio_input_decoder::Mouse::ScrollUp => {
                                        add_scroll(1)
                                    }
                                    virtio_input_decoder::Mouse::ScrollDown => {
                                        add_scroll(2)
                                    }
                                    virtio_input_decoder::Mouse::X(x) => {
                                        add_mouse_x(x);
                                    }
                                    virtio_input_decoder::Mouse::Y(y) => {
                                        add_mouse_y(y);
                                    }
                                }
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    pub fn draw_rect_blend(&mut self, device_idx : usize, rect : Rect, buffer : &[Pixel]) {
        let gpu = self.gpu_device.get_mut(device_idx).unwrap();
        gpu.draw_blend(rect, buffer).unwrap();
    }

    pub fn draw_rect_override(&mut self,device_idx:usize, rect : Rect, buffer : &[Pixel]) {
        let gpu = self.gpu_device.get_mut(device_idx).unwrap();
        gpu.draw_override(rect, buffer).unwrap();
    }

    pub fn invalid(&mut self) {
        for gpu in self.gpu_device.iter_mut() {
            gpu.refresh();
        }
    }
}

pub static mut DEVICE : Option<Device> = None;

pub fn init() {
    unsafe {
        DEVICE = Some(Device::new());
        invalid();
    }
}

pub fn get_device()->&'static mut Device {
    unsafe {
        let mut rt = None;
        if let Some(dev) = &mut DEVICE {
            rt = Some(dev);
        }
        rt.unwrap()
    }
}

// pub fn sync_write(block_idx : usize, buffer : *mut u8, size : u32, offset : usize){
//     let t = slice_from_raw_parts(buffer, size as usize) as *mut [u8];
//     get_device().sync_write(block_idx, offset, size as usize, unsafe {&*t});
// }

// pub fn sync_read(block_idx : usize, buffer : *mut u8, size : u32, offset : usize){
//     let t = slice_from_raw_parts(buffer, size as usize) as *mut [u8];
//     get_device().sync_read(block_idx, offset, size as usize, unsafe {&mut *t});
// }

pub fn gpu_support()->bool {
    get_device().gpu_device.len() > 0
}

/// ## 覆盖绘制
pub fn draw_rect_override(device_idx : usize, rect : Rect, buffer : *mut Pixel){
    let len = (rect.x2 - rect.x1) * (rect.y2 - rect.y1);
    get_device().draw_rect_override(device_idx, rect, 
        unsafe{&*(slice_from_raw_parts(buffer, len as usize))});
}

/// ## 透明度绘制
pub fn draw_rect_blend(device_idx : usize, rect : Rect, buffer : *mut Pixel){
    let len = (rect.x2 - rect.x1) * (rect.y2 - rect.y1);
    get_device().draw_rect_blend(device_idx, rect,
        unsafe{&*(slice_from_raw_parts(buffer, len as usize))});
}

pub fn pending(num : usize){
    get_device().pending(num);
    get_device().handler(num);
}

pub fn invalid(){
    get_device().invalid();
}


use core::ptr::slice_from_raw_parts;

use alloc::prelude::v1::*;
use crate::{filesystem::push_input, memory::get_manager};
use super::{config::{HEIGHT, WIDTH}, input_buffer::{add_key_press, add_key_release, add_mouse_x, add_mouse_y, add_scroll}};
use virtio_input_decoder::Decoder;
use tisu_driver::{BlockDriver, DeviceType, Driver, GraphicDriver, Pixel, Rect, VirtHeader};
use tisu_driver::InterruptOk;