//! # 文件系统
//! 目前只实现了 FAT32 的读取和执行

pub mod operation;
pub mod fat32;
pub mod elf;
pub mod image;
pub mod file_tree;
pub mod file;
pub mod implement;
pub mod interface;