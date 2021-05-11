//! # 标准输入
//!
//! 2021年4月29日 zg

const STDIN_BUFFER_SIZE : usize = 128;

pub struct Stdin {
    buffer : [char;STDIN_BUFFER_SIZE],
    read_idx : usize,
    write_idx : usize,
}

impl Stdin {
    pub const fn new()->Self {
        Self {
            buffer : [' ';STDIN_BUFFER_SIZE],
            read_idx : 0,
            write_idx : 0,
        }
    }

    pub fn pop(&mut self)->Option<char> {
        if self.read_idx != self.write_idx {
            let idx = self.read_idx;
            self.read_idx = (self.read_idx + 1) % STDIN_BUFFER_SIZE;
            Some(self.buffer[idx])
        }
        else {
            None
        }
    }

    pub fn push(&mut self, c : char) {
        self.buffer[self.write_idx] = c;
        self.write_idx = (self.write_idx + 1) % STDIN_BUFFER_SIZE;
    }
}
