//! File trait & inode(dir, file, pipe, stdin, stdout)

extern crate log;

mod inode;
mod stdio;

use crate::mm::UserBuffer;

/// trait File for all file types
pub trait File: Send + Sync {
    /// the file readable?
    fn readable(&self) -> bool;
    /// the file writable?
    fn writable(&self) -> bool;
    /// read from the file to buf, return the number of bytes read
    fn read(&self, buf: UserBuffer) -> usize;
    /// write to the file from buf, return the number of bytes written
    fn write(&self, buf: UserBuffer) -> usize;
}

/// The stat of a inode
#[repr(C)]
#[derive(Debug)]
pub struct Stat {
    /// ID of device containing file
    pub dev: u64,
    /// inode number
    pub ino: u64,
    /// file type and mode
    pub mode: StatMode,
    /// number of hard links
    pub nlink: u32,
    /// unused pad
    pad: [u64; 7],
}

bitflags! {
    /// The mode of a inode
    /// whether a directory or a file
    pub struct StatMode: u32 {
        /// null
        const NULL  = 0;
        /// directory
        const DIR   = 0o040000;
        /// ordinary regular file
        const FILE  = 0o100000;
    }
}

pub use inode::{list_apps, open_file, OSInode, OpenFlags};
pub use stdio::{Stdin, Stdout};

///  创建一个文件的一个硬链接
pub fn link_at(old_name: alloc::string::String, new_name: alloc::string::String) {
    let ri = &inode::ROOT_INODE;
    ri.link_at(old_name, new_name);
}

/// unlink_at
pub fn unlink_at(name: alloc::string::String) {
    let ri = &inode::ROOT_INODE;
    ri.unlink_at(name);
}

/// 通过文件名得到 Stat
pub fn set_file_stat_by_name(name: &alloc::string::String, st: &mut Stat) {
    let ri = &inode::ROOT_INODE;
    st.dev = 0;
    st.ino = ri.get_id_by_name(name) as u64;
    st.nlink = ri.get_nlink_by_name(name);
    // 只有一个目录, 所以 MODE 不可能是 DIR
    st.mode = StatMode::FILE;
}
