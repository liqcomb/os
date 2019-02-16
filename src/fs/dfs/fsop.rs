use alloc::sync::Arc;
use common::bitmap::Bitmap;
use common::consts::SEEK_SET;
use core::mem::{transmute, zeroed};
use dev::Device;
use fs::dfs::vnop::DDirectory;
use fs::dfs::{DINode, DSuperBlock, DFS_MAGIC, DINODE_MAGIC};
use fs::{filesystems_mut, insert, FileSystem, VNode};
use spin::RwLock;

const BLOCK_SIZE: u32 = 512; // Sector size

#[derive(Clone)]
pub struct BlockAlloc {
    block_cnt: u32,
    freemap: Bitmap,
}

impl BlockAlloc {
    pub fn alloc(&mut self) -> Result<u32, ::common::error::Error> {
        // Search for free blocks linearly
        // This could be improved, greatly
        for idx in 1..self.block_cnt {
            // skips the superblock
            if self.freemap.get(idx as usize) == false {
                self.freemap.set(idx as usize);
                return Ok(idx);
            }
        }
        Err(err!(EFULL))
    }

    pub fn free(&mut self, blockno: u32) -> Result<(), ::common::error::Error> {
        if self.freemap.get(blockno as usize) == false {
            panic!("free() called on free block");
        }
        self.freemap.clear(blockno as usize);
        Ok(())
    }
}

#[derive(Clone)]
pub struct DFS {
    allocator: Arc<RwLock<BlockAlloc>>,
    root_node: Arc<VNode>,
}

impl DFS {
    /// Mount the file system
    pub fn mount(dev: &mut Device, option: u64) -> Result<u64, ::common::error::Error> {
        // Read in the super block
        let _ = try!(dev.seek(SEEK_SET, 0));
        let mut block: DSuperBlock = unsafe { zeroed() };
        {
            use core::mem::size_of;
            assert_eq!(size_of::<DSuperBlock>(), 512);
        }
        if try!(dev.read(unsafe { transmute::<&mut DSuperBlock, &mut [u8; 512]>(&mut block) }))
            != 512
        {
            return Err(err!(EIO));
        }

        if block.magic != DFS_MAGIC {
            return Err(err!(EBADFS));
        }

        // Read free bitmap
        let fmblock_cnt = align!(block.blockno, (BLOCK_SIZE * 8)) / (BLOCK_SIZE * 8);
        let mut data: [u8; 512] = [0; 512];
        let mut bitmap = Bitmap::new(block.blockno as usize);
        try!(dev.seek(SEEK_SET, 2 * (BLOCK_SIZE as u64)));
        for i in 0..fmblock_cnt {
            if try!(dev.read(&mut data)) != 512 {
                return Err(err!(EIO));
            }
            bitmap.set_direct(&data, (BLOCK_SIZE * i) as usize);
        }

        // Load Root INode
        let mut inode: DINode = unsafe { zeroed() };
        {
            use core::mem::size_of;
            assert_eq!(size_of::<DINode>(), 512);
        }
        try!(dev.seek(SEEK_SET, BLOCK_SIZE as u64));
        if try!(dev.read(unsafe { transmute::<&mut DINode, &mut [u8; 512]>(&mut inode) })) != 512 {
            return Err(err!(EIO));
        }
        if inode.magic != DINODE_MAGIC {
            return Err(err!(EBADFS));
        }

        let allocator = Arc::new(RwLock::new(BlockAlloc {
            block_cnt: block.blockno,
            freemap: bitmap,
        }));
        let root_vnode = Arc::new(DDirectory::new(&inode, allocator.clone()));
        let fs = DFS {
            allocator: allocator.clone(),
            root_node: root_vnode,
        };

        // Mount this thing to filesystem
        return insert(fs);
    }
}

impl FileSystem for DFS {
    fn sync(&mut self) -> Result<(), ::common::error::Error> {
        unimplemented!();
    }

    fn name(&self) -> &'static str {
        unimplemented!();
    }

    fn root(&mut self) -> Arc<VNode> {
        unimplemented!();
    }

    fn unmount(&mut self) -> Result<(), ::common::error::Error> {
        Err(err!(EINVAL))
    }
}
