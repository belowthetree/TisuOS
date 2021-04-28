use device_buffer::{Cache, CacheBuffer};

use crate::memory::get_manager;

use super::device::get_device;


const BUFFER_SIZE : usize = 1024 * 4096; // 4 MB
pub static mut CACHE : Option<Cache> = None;

pub fn init() {
    unsafe {
        let mut cache = Cache::new(8, BUFFER_SIZE);
        let device = get_device();
        for (idx, blk) in device.block_device.iter_mut().enumerate() {
            let blk = blk.as_mut();
            cache.add_buffer(idx, blk, get_manager());
        }
        
        CACHE = Some(cache);
    }
}

pub fn get_cache()->&'static mut dyn CacheBuffer {
    let mut rt = None;
    unsafe {
        if let Some(cache) = &mut CACHE {
            rt = Some(cache);
        }
        rt.unwrap()
    }
}

pub fn sync_read_buffer(block_idx : usize, data : &mut [u8], st : usize){
    unsafe {
        if let Some(cache) = &mut CACHE {
            cache.read(block_idx, data, st)
        }
    }
}

pub fn sync_write_buffer(block_idx : usize, data : &[u8], st : usize){
    unsafe {
        if let Some(cache) = &mut CACHE {
            cache.write(block_idx, data, st)
        }
    }
}

