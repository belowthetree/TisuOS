//! # 设备控制寄存器结构
//! 
//! 2021年3月28日 zg


#[allow(dead_code)]
pub enum StatusField {
	Acknowledge = 1,
	Driver = 2,
	Failed = 128,
	FeaturesOk = 8,
	DriverOk = 4,
	DeviceNeedsReset = 64,
}

impl StatusField {
    pub fn val32(self) -> u32{
        self as u32
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct VirtHeader {
    magicvalue : u32,
    version : u32,
    deviceid : u32,
    vendorid : u32,
    host_features : u32,
    host_features_sel : u32,
    rev0 : [u32;2],
    guest_features : u32,
    guest_features_sel : u32,
    guest_page_size : u32,
    rev1 : u32,
    queue_sel : u32,
    queue_num_max : u32,
    queue_num : u32,
    queue_align : u32,
    queue_pfn : u32,
    rev2 : [u32;3],
    queue_notify : u32,
    rev3 : [u32;3],
    interrupt_status : u32,
    interrupt_ack : u32,
    rev4 : [u32;2],
    status : u32,
}

impl VirtHeader {
    pub fn set_feature(&mut self, guest_feat : u32)->Result<(), ()> {
        self.status = 0;
        self.status = StatusField::Acknowledge.val32();
        self.status |= StatusField::DriverOk.val32();
        println!("host feature {:x}", self.host_features);
        self.guest_features = self.host_features & guest_feat;
        self.status |= StatusField::FeaturesOk.val32();
        let status_ok = self.status;
        if status_ok & StatusField::FeaturesOk.val32() == 0 {
            println!("Set up block device fail");
            return Err(());
        }
        Ok(())
    }

    pub fn set_ring_size(&mut self, size : u32)->Result<(), ()> {
        if self.queue_num_max < size {
            Err(())
        }
        else {
            self.queue_num = size;
            Ok(())
        }
    }

    pub fn set_pfn(&mut self, sel : u32, addr : *mut VirtQueue) {
        self.queue_sel = sel;
        self.queue_pfn = addr as u32 / PAGE_SIZE as u32;
    }

    pub fn set_page_size(&mut self, size : u32) {
        self.guest_page_size = size;
    }

    pub fn driver_ok(&mut self) {
        self.status = StatusField::DriverOk.val32();
    }

    pub fn notify(&mut self) {
        self.queue_notify = 0;
    }
}


use crate::{memory::{config::PAGE_SIZE}, uart};

use super::queue::VirtQueue;