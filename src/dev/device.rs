use core::marker::{Send, Sync};

pub trait Device: Sync + Send {
    /// Read from the device
    fn read(&mut self, data: &mut [u8]) -> Result<usize, ::common::error::Error>;
    /// Write to the device
    fn write(&mut self, data: &[u8]) -> Result<usize, ::common::error::Error>;
    /// A device must have the ability to do I/O Control
    fn ioctl(&mut self, ops: u64, data: usize) -> Result<usize, ::common::error::Error>;
    /// Seek device
    fn seek(&mut self, whence: u32, offset: u64) -> Result<u64, ::common::error::Error>;
}
