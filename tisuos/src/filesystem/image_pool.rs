use alloc::collections::BTreeMap;

use fs_format::{BMP, Image};
use alloc::string::String;

use super::io::read;

pub static mut IMAGE_POOL : Option<BTreeMap<String, Image>> = None;

pub fn request<'a>(path:String, width:usize, height:usize)->Result<&'a mut Image, ()> {
    unsafe {
        if let Some(pool) = &mut IMAGE_POOL {
            if pool.contains_key(&path) {
                let img = pool.get_mut(&path).unwrap();
                img.resize(width, height);
                return Ok(img);
            }
            else {
                let data = read(path.clone()).unwrap();
                let mut img = BMP::decode(data.to_array(0, data.size)).unwrap();
                img.resize(width, height);
                pool.insert(path.clone(), img);
                return Ok(pool.get_mut(&path).unwrap());
            }
        }
        else {
            let mut pool = BTreeMap::new();
            let data = read(path.clone()).unwrap();
            let img = BMP::decode(data.to_array(0, data.size)).unwrap();
            pool.insert(path.clone(), img);
            IMAGE_POOL = Some(pool);
            return request(path, width, height);
        }
    }
}