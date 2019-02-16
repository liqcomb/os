use core::ops::Drop;
use fs::Stat;

/// VNode represents an in-memory inode
pub trait VNode: Sync + Send {
    /// Read from the node
    fn read(&mut self, data: &mut [u8], offset: u64) -> Result<u64, ::common::error::Error> {
        return Err(err!(EINVAL));
    }
    /// Write to the node
    fn write(&mut self, data: &[u8], offset: u64) -> Result<u64, ::common::error::Error> {
        return Err(err!(EINVAL));
    }
    /// Ioctl
    fn ioctl(&mut self, op: u32, data: usize) -> Result<u64, ::common::error::Error> {
        return Err(err!(EINVAL));
    }

    /// Create file
    fn creat(&mut self, name: &str, option: u32) -> Result<(), ::common::error::Error> {
        return Err(err!(EINVAL));
    }
    /// Make directory
    fn mkdir(&mut self, name: &str) -> Result<(), ::common::error::Error> {
        return Err(err!(EINVAL));
    }
    /// Remove directory
    fn rmdir(&mut self, name: &str) -> Result<(), ::common::error::Error> {
        return Err(err!(EINVAL));
    }

    /// Get file stat
    fn stat(&self) -> Result<Stat, ::common::error::Error> {
        return Err(err!(EINVAL));
    }
    /// Chmod
    fn chmod(&mut self, mode: u16) -> Result<(), ::common::error::Error> {
        return Err(err!(EINVAL));
    }
    /// Chown
    fn chown(&mut self, owner: u32) -> Result<(), ::common::error::Error> {
        return Err(err!(EINVAL));
    }
    /// Chgrp
    fn chgrp(&mut self, group: u32) -> Result<(), ::common::error::Error> {
        return Err(err!(EINVAL));
    }

    /// Reclaim
    fn reclaim(&mut self) -> Result<(), ::common::error::Error>;
}
