//! # 块设备功能封装
//! 
//! 2021年3月30日 zg

use core::mem::size_of;

use crate::{memory::{alloc_kernel_page, config::PAGE_SIZE, free, alloc}, sync::mutex::{Bool, Mutex}};

use super::{header::VirtHeader,
	queue::{DescFlag, Header, VIRTIO_BLK_F_RO, VIRTIO_BLK_T_IN, VIRTIO_BLK_T_OUT,
		VIRTIO_RING_SIZE, VirtQueue}};


struct Request {
	pub header: Header,
	pub data:   *mut u8,
	pub status: u8,
	pub waiter_id : usize,
	pub lock : Mutex,
}

impl Request {
	pub fn new(buffer:*mut u8,offset:usize,write:bool)->*mut Self {
		let addr = alloc(size_of::<Request>(), true).unwrap();
		let rt = addr as *mut Request;
		let rq = unsafe {&mut *rt};
		rq.header.blktype = if write {VIRTIO_BLK_T_OUT} else {VIRTIO_BLK_T_IN};
		rq.header.sector = (offset / 512) as u64;
		rq.header.reserved = 0;
		rq.data = buffer;
		rq.status = 111;

		rt
	}
}

pub struct Block {
	header : &'static mut VirtHeader,
	queue : &'static mut VirtQueue,
	pub pin_idx : usize,
	pub int : Bool,
}

impl Clone for Block {
    fn clone(&self) -> Self {
        Block{
			header : unsafe{&mut *(self.header as *const VirtHeader as usize as *mut VirtHeader)},
			queue : unsafe{&mut *(self.queue as *const VirtQueue as usize as *mut VirtQueue)},
			int : Bool::new(),
            pin_idx: self.pin_idx,
		}
    }
}

impl Block {
	pub fn new(idx : usize, header : *mut VirtHeader)->Self {
		let num = (size_of::<VirtQueue>() + PAGE_SIZE - 1) / PAGE_SIZE;
		let queue = alloc_kernel_page(num).unwrap() as *mut VirtQueue;
		let header = unsafe {&mut *header};
		header.set_feature(!(1 << VIRTIO_BLK_F_RO)).unwrap();
		header.set_ring_size(VIRTIO_RING_SIZE as u32).unwrap();
		header.set_page_size(PAGE_SIZE as u32);
		header.set_pfn(0, queue);
		header.driver_ok();
		let rt = Self {
			header : header,
			queue : unsafe {&mut *queue},
			int : Bool::new(),
		    pin_idx: idx,
		};
		rt
	}

	pub fn sync_write(&mut self, buffer : *mut u8, size : u32, offset : usize) {
		let rq = Request::new(buffer, offset, true);
		let header = unsafe {&(*rq).header as *const Header};
		let status = unsafe {&(*rq).status as *const u8};
		let mut flag = DescFlag::Next as u16;
		self.queue.add_avail();
		self.queue.add_desc(header as u64,size_of::<Header>() as u32,flag);
		self.queue.add_desc(buffer as u64, size, flag);
		flag = DescFlag::Write as u16;
		self.queue.add_desc(status as u64, 1, flag);
		self.header.notify();
		unsafe {
			(*rq).lock.lock();
		}
		self.header.notify();
		unsafe {
			(*rq).lock.lock();
			(*rq).lock.unlock();
		}
		free(rq as *mut u8);
	}

	pub fn sync_read(&mut self, buffer : *mut u8, size : u32, offset : usize) {
		let rq = Request::new(buffer, offset, false);
		let header = unsafe {&(*rq).header as *const Header};
		let status = unsafe {&(*rq).status as *const u8};
		let mut flag = DescFlag::Next as u16;
		self.queue.add_avail();
		self.queue.add_desc(header as u64,size_of::<Header>() as u32,flag);
		flag |= DescFlag::Write as u16;
		self.queue.add_desc(buffer as u64, size, flag);
		flag = DescFlag::Write as u16;
		self.queue.add_desc(status as u64, 1, flag);
		unsafe {
			(*rq).lock.lock();
		}
		self.header.notify();
		unsafe {
			(*rq).lock.lock();
			(*rq).lock.unlock();
		}
		free(rq as *mut u8);
	}

	pub fn interrupt_handler(&mut self) {
		if !self.int.pop() {return;}

		while self.queue.is_pending() {
			let elem = self.queue.next_elem();
			let rq = self.queue.desc[elem.id as usize].addr as *mut Request;
			let pid = unsafe {(*rq).waiter_id};
			if pid > 0{
				// wake_up(pid);
				free(rq as *mut u8);
			}
			else {
				unsafe {(*rq).lock.unlock()}
			}
		}
	}

	pub fn pending(&mut self) {
		self.int.set_true();
	}
}
