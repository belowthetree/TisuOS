#![allow(dead_code)]
//! # Block device
//! 管理块设备，开机扫描所有块设备并记录
//! 2020年12月16日 zg
use core::{mem::size_of};
use alloc::{prelude::v1::*};

use device::{StatusField, VIRTIO_BLK_T_IN, VIRTIO_BLK_T_OUT, Descriptor, VIRTIO_RING_SIZE
		, VIRTIO_BLK_F_RO, Queue, Request, Header, Status};
use super::{device::{self, Offset}};

// pub const VIRTIO_BLK_F_SIZE_MAX: u32 = 1;
// pub const VIRTIO_BLK_F_SEG_MAX: u32 = 2;
// pub const VIRTIO_BLK_F_GEOMETRY: u32 = 4;
// pub const VIRTIO_BLK_F_BLK_SIZE: u32 = 6;
// pub const VIRTIO_BLK_F_FLUSH: u32 = 9;
// pub const VIRTIO_BLK_F_TOPOLOGY: u32 = 10;
// pub const VIRTIO_BLK_F_CONFIG_WCE: u32 = 11;
// pub const VIRTIO_BLK_F_DISCARD: u32 = 13;
// pub const VIRTIO_BLK_F_WRITE_ZEROES: u32 = 14;

/// ## BlockDevice
/// 记录此设备所在的针脚等信息
pub struct BlockDevice{
	pub pin_idx : usize,
	pub queue_idx : usize,
	pub used_idx : usize,
	pub queue : *mut Queue,
	pub ptr : *mut u32,
	pub int : Bool,
}

impl BlockDevice {
	pub fn new(pin_idx : usize, queue : *mut Queue, ptr : *mut u32) ->Self{
		BlockDevice{
			pin_idx : pin_idx,
			queue_idx : 0,
			used_idx : 0,
			queue : queue,
			ptr : ptr,
			int : Bool::new(),
		}
	}
	pub fn add_idx(&mut self) ->usize {
		self.queue_idx = (self.queue_idx + 1) % VIRTIO_RING_SIZE;
		self.queue_idx
	}
	pub fn next_idx(&self) ->u16 {
		((self.queue_idx + 1) % VIRTIO_RING_SIZE) as u16
	}
	/// 块设备的操作依赖于请求结构体（Request）
	/// Request 会被分尸装进描述符里，大致是分成：头、数据、状态三部分放入三个描述符里
	pub fn operation(&mut self, buffer : *mut u8, size : u32,
			offset : usize, write : bool, request : *mut Request){
		unsafe {
			let sector = offset / 512;
			self.add_idx();
			let head_idx = self.queue_idx as u16;
			let desc = Descriptor { addr:  &(*request).header as *const Header as u64,
									len:   size_of::<Header>() as u32,
									flags: device::VIRTIO_DESC_F_NEXT,
									next:  self.next_idx(), };
			(*self.queue).desc[self.queue_idx] = desc;
			(*request).header.sector = sector;
			(*request).header.blktype = if true == write {VIRTIO_BLK_T_OUT}else{VIRTIO_BLK_T_IN};
			
			(*request).data.data = buffer;
			(*request).header.reserved = 0;
			(*request).status.status = 111;
			self.add_idx();
			let desc = Descriptor { addr:  buffer as u64,
									len:   size,
									flags: device::VIRTIO_DESC_F_NEXT
											| if false == write {
												device::VIRTIO_DESC_F_WRITE
											}
											else {
												0
											},
									next:  self.next_idx(), };
			(*self.queue).desc[self.queue_idx] = desc;
			let desc = Descriptor { addr:  &(*request).status as *const Status as u64,
									len:   size_of::<Status>() as u32,
									flags: device::VIRTIO_DESC_F_WRITE,
									next:  0, };
			self.add_idx();
			(*self.queue).desc[self.queue_idx] = desc;
			(*self.queue).avail.ring[(*self.queue).avail.idx as usize % VIRTIO_RING_SIZE] = head_idx;
			(*self.queue).avail.idx = (*self.queue).avail.idx.wrapping_add(1);
			
			self.ptr.add(Offset::QueueNotify.scale32()).write_volatile(0);
		}
	}
	/// 块设备完成数据传输后会在 Queue 的 used 中将 idx 加一
	pub fn interrupt_handler(&mut self){
		if !self.int.get() {
			return;
		}
		
		unsafe{
			let queue = &*self.queue;
			while self.used_idx as u16 != queue.used.idx {
				let ref elem = queue.used.ring[self.used_idx % VIRTIO_RING_SIZE];
				self.used_idx = self.used_idx.wrapping_add(1);
				let rq = queue.desc[elem.id as usize].addr as *mut Request;
				let pid = (*rq).waiter_pid;

				if pid > 0{
					wake_up(pid);
					free(rq as *mut u8);
				}
				else {
					(*rq).lock.unlock();
				}
			}
		}
	}
}

pub fn init(){
	unsafe {
		BLOCKS = Some(Vec::<BlockDevice>::new());
	}
}
pub fn init_block(ptr : *mut u32, idx : usize) -> bool {
    unsafe {
        // 1. 重设
		ptr.add(Offset::Status.scale32()).write_volatile(0);
		let mut status_bits = StatusField::Acknowledge.val32();
		// 2. 在 Status 中设置 ACKNOWLEDGE 位 
		ptr.add(Offset::Status.scale32()).write_volatile(status_bits);
		// 3. 设置 DRIVER 位
		status_bits |= StatusField::DriverOk.val32();
		ptr.add(Offset::Status.scale32()).write_volatile(status_bits);
		// 4. 读取 feature bits, 写入子集，这里除了只读其它的都写入
		let host_features = ptr.add(Offset::HostFeatures.scale32()).read_volatile();
        let guest_features = host_features & !(1 << VIRTIO_BLK_F_RO);
		ptr.add(Offset::GuestFeatures.scale32()).write_volatile(guest_features);
		// 5. 设置 FEATURES_OK
		status_bits |= StatusField::FeaturesOk.val32();
		ptr.add(Offset::Status.scale32()).write_volatile(status_bits);
		// 6. 再次读取 Status，确保设置正确
        let status_ok = ptr.add(Offset::Status.scale32()).read_volatile();
        if status_ok & StatusField::FeaturesOk.val32() == 0 {
            println!("Set up block device fail");
            return false;
        }
		// 7. 读取并设置 Queue 的长度，不能超过 Queue num max 的值
		let qnmax = ptr.add(Offset::QueueNumMax.scale32()).read_volatile();
		ptr.add(Offset::QueueNum.scale32()).write_volatile(VIRTIO_RING_SIZE as u32);
		if VIRTIO_RING_SIZE as u32 > qnmax {
			print!("queue size fail...");
			return false;
        }
        
		let num_pages = (size_of::<Queue>() + page::PAGE_SIZE - 1) / page::PAGE_SIZE;
		// 选择要设置的 Queue 的下标，从 0 开始
		ptr.add(Offset::QueueSel.scale32()).write_volatile(0);
        let queue_ptr = alloc_kernel_page(num_pages) as *mut Queue;
        // 8. 设置页面大小
		let queue_pfn = queue_ptr as u32;
		ptr.add(Offset::GuestPageSize.scale32()).write_volatile(page::PAGE_SIZE as u32);
		// 9. 设置 Queue 所在页面号码
		ptr.add(Offset::QueuePfn.scale32()).write_volatile(queue_pfn / page::PAGE_SIZE as u32);
		// 10. 设置 DRIVER_OK
		status_bits |= StatusField::DriverOk.val32();
		ptr.add(Offset::Status.scale32()).write_volatile(status_bits);
		
		if let Some(block) = &mut BLOCKS{
			block.push(BlockDevice::new(idx, queue_ptr, ptr));
		}

        true
    }
}

pub static mut BLOCKS : Option<Vec<BlockDevice>> = None;
/// 异步读写使用进程实现
pub fn async_write(block_idx : usize, buffer : *mut u8, size : u32, offset : usize, handler : usize){
	unsafe {
		if let Some(block) = &mut BLOCKS{
			assert!(block_idx < block.len());
			let request = alloc(size_of::<Request>(), true).unwrap() as *mut Request;
			if handler == 0{}
			else if let Some(mut p) = process::create_process(handler as usize, true){
				p.sleep();
				(*request).waiter_pid = p.pid;
				process::add_process(p);
			}
			else{
				println!("no write process");
			}
			block[block_idx].operation(buffer, size, offset, true, request);
		}
	}
}
pub fn async_read(block_idx : usize, buffer : *mut u8, size : u32, offset : usize, handler : fn()){
	unsafe {
		if let Some(block) = &mut BLOCKS{
			assert!(block_idx < block.len());
			let request = alloc(size_of::<Request>(), true).unwrap() as *mut Request;
			
			if let Some(mut p) = process::create_process(handler as usize, true){
				p.sleep();
				(*request).waiter_pid = p.pid;
				process::add_process(p);
			}
			else{
				println!("no read process");
			}
			
			block[block_idx].operation(buffer, size, offset, false, request);
		}
	}
}

/// 同步读写仅使用同步锁实现
pub fn sync_write(block_idx : usize, buffer : *mut u8, size : u32, offset : usize){
	unsafe {
		if let Some(block) = &mut BLOCKS{
			assert!(block_idx < block.len());
			let request = alloc(size_of::<Request>(), true).unwrap() as *mut Request;
			(*request).lock.lock();
			block[block_idx].operation(buffer, size, offset, true, request);
			(*request).lock.lock();
			(*request).lock.unlock();
			free(request as *mut u8);
		}
	}
}
pub fn sync_read(block_idx : usize, buffer : *mut u8, size : u32, offset : usize){
	unsafe{
		if let Some(block) = &mut BLOCKS{
			assert!(block_idx < block.len());
			let request = alloc(size_of::<Request>(), true).unwrap() as *mut Request;
			assert!(!request.is_null());
			(*request).lock.lock();
			block[block_idx].operation(buffer, size, offset, false, request);
			(*request).lock.lock();
			(*request).lock.unlock();
			free(request as *mut u8);
		}
	}
}

pub fn interrupt_handler(pin_num : usize){
	unsafe {
		if let Some(block) = &mut BLOCKS{
			for b in block{
				if b.pin_idx == pin_num{
					// b.interrupt_handler();
					b.int.set_true();
					break;
				}
			}
		}
	}
}

pub fn run_interrupt() {
	unsafe {
		if let Some(block) = &mut BLOCKS{
			for b in block{
				b.interrupt_handler();
			}
		}
	}
}

use crate::{memory::page, sync::Bool, task::process::wake_up, uart};
use crate::memory::page::{alloc_kernel_page};
use crate::memory::allocator::{alloc, free};
use crate::{task::process};