mod inode;
mod stdio;

use crate::mm::UserBuffer;

/// The common abstraction of all IO resources
pub trait File: Send + Sync {
    fn readable(&self) -> bool;
    fn writable(&self) -> bool;
    fn read(&self, buf: UserBuffer) -> usize;
    fn write(&self, buf: UserBuffer) -> usize;
    fn stat(&self) -> Stat;
}

pub use easy_fs::Stat;

pub use inode::{linkat, list_apps, open_file, unlinkat, OSInode, OpenFlags};
pub use stdio::{Stdin, Stdout};
