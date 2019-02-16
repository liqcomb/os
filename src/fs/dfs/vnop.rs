use alloc::sync::Arc;
use fs::dfs::fsop::BlockAlloc;
use fs::dfs::DINode;
use fs::{Stat, VNode};
use spin::RwLock;

pub struct DFile {
    allocator: Arc<RwLock<BlockAlloc>>,
}

impl VNode for DFile {
    fn stat(&self) -> Result<Stat, ::common::error::Error> {
        unimplemented!();
    }

    fn read(&mut self, data: &mut [u8], offset: u64) -> Result<u64, ::common::error::Error> {
        unimplemented!();
    }

    fn write(&mut self, data: &[u8], offset: u64) -> Result<u64, ::common::error::Error> {
        unimplemented!();
    }

    fn chmod(&mut self, mode: u16) -> Result<(), ::common::error::Error> {
        unimplemented!();
    }

    fn chown(&mut self, owner: u32) -> Result<(), ::common::error::Error> {
        unimplemented!();
    }

    fn chgrp(&mut self, group: u32) -> Result<(), ::common::error::Error> {
        unimplemented!();
    }

    fn reclaim(&mut self) -> Result<(), ::common::error::Error> {
        unimplemented!();
    }
}

impl DFile {
    pub fn new(inode: &DINode, allocator: Arc<RwLock<BlockAlloc>>) -> Self {
        unimplemented!();
    }
}

impl Drop for DFile {
    fn drop(&mut self) {
        match self.reclaim() {
            Err(e) => panic!(e.message()),
            _ => {}
        };
    }
}

pub struct DDirectory {
    allocator: Arc<RwLock<BlockAlloc>>,
}

impl VNode for DDirectory {
    fn stat(&self) -> Result<Stat, ::common::error::Error> {
        unimplemented!();
    }

    fn creat(&mut self, name: &str, option: u32) -> Result<(), ::common::error::Error> {
        unimplemented!();
    }

    fn mkdir(&mut self, name: &str) -> Result<(), ::common::error::Error> {
        unimplemented!();
    }

    fn rmdir(&mut self, name: &str) -> Result<(), ::common::error::Error> {
        unimplemented!();
    }

    fn chmod(&mut self, mode: u16) -> Result<(), ::common::error::Error> {
        unimplemented!();
    }

    fn chown(&mut self, owner: u32) -> Result<(), ::common::error::Error> {
        unimplemented!();
    }

    fn chgrp(&mut self, group: u32) -> Result<(), ::common::error::Error> {
        unimplemented!();
    }

    fn reclaim(&mut self) -> Result<(), ::common::error::Error> {
        unimplemented!();
    }
}

impl Drop for DDirectory {
    fn drop(&mut self) {
        match self.reclaim() {
            Err(e) => panic!(e.message()),
            _ => {}
        };
    }
}

impl DDirectory {
    pub fn new(inode: &DINode, allocator: Arc<RwLock<BlockAlloc>>) -> Self {
        unimplemented!();
    }
}
