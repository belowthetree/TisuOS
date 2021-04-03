#![allow(dead_code)]
//! # Block device
//! 管理块设备，开机扫描所有块设备并记录
//! 2020年12月16日 zg

#![allow(dead_code)]

pub static mut BLOCKS : Option<Vec<BlockDevice>> = None;
/// 异步读写使用进程实现
pub fn async_write(block_idx : usize, buffer : *mut u8, size : u32, offset : usize, handler : usize){
	unsafe {
		if let Some(block) = &mut BLOCKS{
			assert!(block_idx < block.len());
			let request = alloc(size_of::<Request>(), true).unwrap() as *mut Request;
			if handler == 0{}
			else{
				let id = branch(handler);
				(*request).waiter_id = id;
			}
			block[block_idx].operation(buffer, size, offset, true, request);
		}
	}
}
pub fn async_read(block_idx : usize, buffer : *mut u8, size : u32, offset : usize, handler : usize){
	unsafe {
		if let Some(block) = &mut BLOCKS{
			assert!(block_idx < block.len());
			let request = alloc(size_of::<Request>(), true).unwrap() as *mut Request;
			
			if handler == 0{}
			else{
				let id = branch(handler);
				(*request).waiter_id = id;
			}
			
			block[block_idx].operation(buffer, size, offset, false, request);
		}
	}
}

