#![allow(dead_code)]
//! # Device
//! 虚拟IO设备管理
//! 开机扫描 0x1000_1000 到 0x1000_8000 的内存地址，查找存在的设备并初始化、记录
//! 2020年11月 zg

pub static VIRTIO_START_ADDR : usize = 0x1000_1000;
pub static VIRTIO_END_ADDR : usize = 0x1000_8000;
static VIRTIO_STEP_SIZE : usize = 0x1000;
static VIRTIO_VAL : u32 = 0x74726976;

pub const VIRTIO_DESC_F_NEXT: u16 = 1;
pub const VIRTIO_DESC_F_WRITE: u16 = 2;
pub const VIRTIO_DESC_F_INDIRECT: u16 = 4;
pub const VIRTIO_AVAIL_F_NO_INTERRUPT: u16 = 1;
pub const VIRTIO_USED_F_NO_NOTIFY: u16 = 1;

// Type values
pub const VIRTIO_BLK_T_IN: u32 = 0;
pub const VIRTIO_BLK_T_OUT: u32 = 1;
pub const VIRTIO_BLK_T_FLUSH: u32 = 4;
pub const VIRTIO_BLK_T_DISCARD: u32 = 11;
pub const VIRTIO_BLK_T_WRITE_ZEROES: u32 = 13;
pub const VIRTIO_BLK_F_RO: u32 = 5;
pub const VIRTIO_RING_SIZE : usize = 1 << 7;
pub const VIRTIO_F_RING_EVENT_IDX : u32 = 29;
#[repr(C)]
pub struct Queue {
	pub desc:  [Descriptor; VIRTIO_RING_SIZE],
	pub avail: Available,
	pub padding0: [u8; PAGE_SIZE - size_of::<Descriptor>() * VIRTIO_RING_SIZE - size_of::<Available>()],
	pub used:     Used,
}

#[repr(C)]
#[derive(Debug)]
pub struct Descriptor {
	pub addr:  u64,
	pub len:   u32,
	pub flags: u16,
	pub next:  u16,
}
/// ## Request
/// waiter pid 用于异步读写，记录等待的进程
/// lock 用于同步读写
#[repr(C)]
pub struct Request {
	pub header: Header,
	pub data:   Data,
	pub status: Status,
	pub waiter_pid : usize,
	pub lock : sync::Mutex,
}
#[repr(C)]
pub struct UsedElem {
	pub id:  u32,
	pub len: u32,
}
#[repr(C)]
pub struct Used {
	pub flags: u16,
	pub idx:   u16,
	pub ring:  [UsedElem; VIRTIO_RING_SIZE],
	pub event: u16,
}
#[repr(C)]
pub struct Available {
	pub flags: u16,
	pub idx:   u16,
	pub ring:  [u16; VIRTIO_RING_SIZE],
	pub event: u16,
}
#[repr(C)]
pub struct Header {
	pub blktype:  u32,
	pub reserved: u32,
	pub sector:   usize,
}
#[repr(C)]
pub struct Data {
	pub data: *mut u8,
}
#[repr(C)]
pub struct Status {
	pub status: u8,
}

#[derive(Copy, Clone)]
#[repr(usize)]
pub enum DeviceType {
    None = 0,
    Network = 1,
    Block = 2,
    Console = 3,
    Entropy = 4,
    Gpu = 16,
    Input = 18,
    Memory = 24,
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

pub static mut DEVICE : [DeviceType;9] = [DeviceType::None;9];

pub fn init(){
    block_device::init();
    input_device::init();
    gpu_device::init();
    for i in (VIRTIO_START_ADDR..=VIRTIO_END_ADDR).step_by(VIRTIO_STEP_SIZE){
        let ptr = i as *mut u32;
        let magic;
        let device_id;
        unsafe {
            magic = ptr.read_volatile();
            device_id = ptr.add(2).read_volatile();
        }
        println!("address 0x{:x}", i);
        if magic == VIRTIO_VAL {
            let idx = (i - VIRTIO_START_ADDR) / VIRTIO_STEP_SIZE + 1;
            match device_id {
                1 => {
                    println!("Network device");
                }
                2 => {
                    println!("Block device");
                    unsafe {
                        DEVICE[idx] = 
                            DeviceType::Block;
                    }
                    block_device::init_block(ptr, idx);
                }
                3 => {
                    println!("Console device");
                }
                18 => {
                    println!("Input device");
                    unsafe {
                        DEVICE[idx] = 
                            DeviceType::Input;
                    }
                    // input_device::init();
                    // input::setup_input_device(ptr);
                    input_device::setup_input_device(ptr, idx);
                }
                16 => {
                    println!("GPU device {}", idx);
                    unsafe {
                        DEVICE[idx] = 
                            DeviceType::Gpu;
                    }
                    gpu_device::init_gpu(ptr, idx);
                    //gpu::setup_gpu_device(ptr);
                }
                0 => {
                    println!("Device not connected");
                },
                _ => {
                }
            }
        }
    }
}

pub fn interrupt_handler(num : usize){
    match unsafe {DEVICE[num]} {
        DeviceType::Block => {
            block_device::interrupt_handler(num);
        }
        DeviceType::Input => {
            input_device::interrupt_handler(num);
        }
        DeviceType::Gpu => {
            gpu_device::pending(num);
        }
        _ => {println!("No device {}", num)}
    }
}

use core::mem::size_of;

use crate::{sync, uart};
use crate::memory::page::{PAGE_SIZE};
use super::{block_device, gpu_device, input_device};