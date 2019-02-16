use alloc::sync::Arc;
use dev::Device;
use fs::VNode;

/// Filesystem trait
pub trait FileSystem: Sync + Send {
    /// Synchronize
    fn sync(&mut self) -> Result<(), ::common::error::Error>;
    /// Get file system name
    fn name(&self) -> &'static str;
    /// Get the root VNode
    fn root(&mut self) -> Arc<VNode>;
    /// Unmount filesystem
    fn unmount(&mut self) -> Result<(), ::common::error::Error>;
}
