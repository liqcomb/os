//! DFS - Dumb FileSystem
//! Supports only a handful of operations
//!
//! | Super Block | 0
//! | Root INode  | 1
//! | Free Map 0  | 2
//! | Free Map 1  | 3
//! | ........... | 4
//! | INode/Data  | 5
//!

mod fsop;
mod vnop;

const DFS_MAGIC: u32 = 0xDEADF5F5;
const DINODE_MAGIC: u32 = 0xDEEAC0DE;

#[repr(C, packed)]
pub struct DSuperBlock {
    magic: u32,
    blockno: u32,
    _unused: [u32; 126],
}

#[repr(C, packed)]
pub struct DINode {
    magic: u32,
    ntype: u32,
    owner: u32,
    group: u32,
    lastwrite: u32,
    access: u16,
    refcnts: u16,
    name: [u8; 16],
    references: [u32; 118], // ref
}

/// Initialize and mount the filesystem
pub fn init() {}
