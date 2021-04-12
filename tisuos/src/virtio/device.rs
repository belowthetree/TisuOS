#![allow(dead_code)]
//! # Device
//! 虚拟IO设备管理
//! 开机扫描 0x1000_1000 到 0x1000_8000 的内存地址，查找存在的设备并初始化、记录
//! 2020年11月 zg

pub static VIRTIO_START_ADDR : usize = 0x1000_1000;
pub static VIRTIO_END_ADDR : usize = 0x1000_8000;
static VIRTIO_STEP_SIZE : usize = 0x1000;
static VIRTIO_VAL : u32 = 0x74726976;


/// ## Request
/// waiter pid 用于异步读写，记录等待的进程
/// lock 用于同步读写
#[repr(C)]
pub struct Request {
	pub header: Header,
	pub data:   *mut u8,
	pub status: u8,
	pub waiter_id : usize,
	pub lock : Mutex,
}

#[derive(Copy, Clone)]
#[repr(usize)]
pub enum DeviceType {
    Unknown = 0,
    Network = 1,
    Block = 2,
    Console = 3,
    Entropy = 4,
    Gpu = 16,
    Input = 18,
    Memory = 24,
}

impl DeviceType {
    pub fn from(num : usize)->Self {
        match num {
            2 => {DeviceType::Block}
            16 => {DeviceType::Gpu}
            18 => {DeviceType::Input}
            _ => {DeviceType::Unknown}
        }
    }
}

#[allow(dead_code)]
#[repr(usize)]
pub enum Offset {
    MagicValue = 0x000,
    Version = 0x004,
    DeviceId = 0x008,
    VendorId = 0x00c,
    HostFeatures = 0x010,
    HostFeaturesSel = 0x014,
    GuestFeatures = 0x020,
    GuestFeaturesSel = 0x024,
    GuestPageSize = 0x028,
    QueueSel = 0x030,
    QueueNumMax = 0x034,
    QueueNum = 0x038,
    QueueAlign = 0x03c,
    QueuePfn = 0x040,
    QueueNotify = 0x050,
    InterruptStatus = 0x060,
    InterruptAck = 0x064,
    Status = 0x070,
    Config = 0x100,
}

#[allow(dead_code)]
pub enum StatusField {
	Acknowledge = 1,
	Driver = 2,
	Failed = 128,
	FeaturesOk = 8,
	DriverOk = 4,
	DeviceNeedsReset = 64,
}

impl Offset {
    pub fn scale32(self) -> usize {
        self.val() / 4
    }

    pub fn val(self) -> usize{
        self as usize
    }
}

impl StatusField {
    pub fn val32(self) -> u32{
        self as u32
    }
}

pub struct Device {
    pub block_device : Vec<Block>,
    pub gpu_device : Vec<GPU>,
    pub input_device : Vec<InputDevice>,
    dtype : [DeviceType;9],
}

impl Device {
    pub fn new()->Self {
        Self {
            block_device : vec![],
            gpu_device : vec![],
            input_device : vec![],
            dtype : [DeviceType::Unknown;9],
        }
    }

    pub fn add_device(&mut self, header : *mut VirtHeader, pin_idx : usize, id : u32) {
        match DeviceType::from(id as usize) {
            DeviceType::Block => {
                self.dtype[pin_idx] = DeviceType::Block;
                self.block_device.push(Block::new(pin_idx, header));
            }
            DeviceType::Gpu => {
                self.dtype[pin_idx] = DeviceType::Block;
                self.gpu_device.push(GPU::new(pin_idx, header, 640, 480))
            }
            DeviceType::Input => {
                self.dtype[pin_idx] = DeviceType::Input;
                self.input_device.push(InputDevice::new(pin_idx, header));
            }
            _ => {}
        }
    }

    pub fn sync_read(&mut self,block_idx :usize,buffer : *mut u8, size : u32, offset : usize) {
        let blk = self.block_device.get_mut(block_idx).unwrap();
        blk.sync_read(buffer, size, offset);
    }

    pub fn sync_write(&mut self,block_idx :usize,buffer : *mut u8, size : u32, offset : usize) {
        let blk = self.block_device.get_mut(block_idx).unwrap();
        blk.sync_write(buffer, size, offset);
    }

    pub fn pending(&mut self, pin_idx : usize) {
        match self.dtype[pin_idx] {
            DeviceType::Block => {
                for blk in self.block_device.iter_mut() {
                    if blk.pin_idx == pin_idx {
                        blk.pending();
                        break;
                    }
                }
            }
            DeviceType::Gpu => {
                for gpu in self.gpu_device.iter_mut() {
                    if gpu.pin_idx == pin_idx {
                        gpu.pending();
                        break;
                    }
                }
            }
            DeviceType::Input => {
                for input in self.input_device.iter_mut() {
                    input.pending();
                }
            }
            _ => {}
        }
    }

    pub fn handler(&mut self) {
        for blk in self.block_device.iter_mut() {
            blk.interrupt_handler();
        }
        for gpu in self.gpu_device.iter_mut() {
            gpu.interrupt_handler();
        }
    }

    pub fn draw_rect_blend(&mut self,device_idx : usize,rect : Rect, color_buffer : *mut Pixel) {
        let gpu = self.gpu_device.get_mut(device_idx).unwrap();
        gpu.draw_rect_blend(rect, color_buffer);
    }

    pub fn draw_rect_override(&mut self,device_idx:usize,rect:Rect,color_buffer : *mut Pixel) {
        let gpu = self.gpu_device.get_mut(device_idx).unwrap();
        gpu.draw_rect_override(rect, color_buffer);
    }

    pub fn invalid(&mut self) {
        for gpu in self.gpu_device.iter_mut() {
            gpu.refresh();
        }
    }
}

pub static mut DEVICES : Option<Device> = None;
pub static mut DEVICE : [DeviceType;9] = [DeviceType::Unknown;9];

pub fn init(){
    unsafe {
        DEVICES = Some(Device::new());
    }

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
            unsafe {
                if let Some(dev) = &mut DEVICES {
                    dev.add_device(header as *mut VirtHeader, idx, device_id);
                }
            }
        }
    }
}

pub fn sync_write(block_idx : usize, buffer : *mut u8, size : u32, offset : usize){
	unsafe {
		if let Some(dev) = &mut DEVICES{
            dev.sync_write(block_idx, buffer, size, offset);
		}
	}
}

pub fn sync_read(block_idx : usize, buffer : *mut u8, size : u32, offset : usize){
	unsafe{
		if let Some(dev) = &mut DEVICES{
            dev.sync_read(block_idx, buffer, size, offset);
		}
	}
}

pub fn gpu_support()->bool {
    unsafe {
        if let Some(dev) = &mut DEVICES {
            dev.gpu_device.len() > 0
        }
        else {false}
    }
}

/// ## 覆盖绘制
pub fn draw_rect_override(device_idx : usize, rect : Rect, color_buffer : *mut Pixel){
    unsafe {
        if let Some(dev) = &mut DEVICES {
            dev.draw_rect_override(device_idx, rect, color_buffer);
        }
    }
}

/// ## 透明度绘制
pub fn draw_rect_blend(device_idx : usize, rect : Rect, color_buffer : *mut Pixel){
    unsafe {
        if let Some(dev) = &mut DEVICES {
            dev.draw_rect_blend(device_idx, rect, color_buffer);
        }
    }
}

pub fn pending(num : usize){
    unsafe {
        if let Some(dev) = &mut DEVICES {
            dev.pending(num);
        }
    }
}

pub fn invalid(){
    unsafe {
        if let Some(dev) = &mut DEVICES {
            dev.invalid();
        }
    }
}

pub fn run_interrupt(){
    loop {
        unsafe {
            asm!("wfi"::::"volatile");
            if let Some(dev) = &mut DEVICES {
                dev.handler();
            }
        }
    }
}


use alloc::{prelude::v1::*};
use crate::{libs::{graphic::Pixel, shape::Rect}, sync::{mutex::Mutex}};
use super::{block::Block, gpu::GPU, header:: VirtHeader,
    input::input::{InputDevice}, queue::{Header}};