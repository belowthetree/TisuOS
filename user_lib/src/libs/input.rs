use virtio_input_decoder::Key;

use crate::libs::syscall::{sys_key_press, sys_key_release, sys_mouse_position, sys_mouse_scroll};

use super::graphic::ScalePoint;

pub fn get_key_release()->Option<Key> {
    let key = sys_key_release();
    Key::from_code(key).ok()
}

pub fn get_key_press()->Option<Key> {
    Key::from_code(sys_key_press()).ok()
}

pub fn get_scroll()->usize {
    sys_mouse_scroll()
}

pub fn get_mouse_position()->ScalePoint {
    let (x, y) = sys_mouse_position();
    ScalePoint::new(x, y)
}
