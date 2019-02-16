use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use spin::{Once, RwLock, RwLockReadGuard, RwLockWriteGuard};

mod dfs;
mod entity;
mod fs;

pub use self::entity::VNode;
pub use self::fs::FileSystem;

pub struct Stat {
    attribute: u32,
    owner: u32,
    group: u32,
    lastwrite: u32,
    access: u16,
}

type FsMap = BTreeMap<u64, Box<fs::FileSystem>>;
const MAX_FS: u64 = 8;

pub static FILESYSTEM_MAP: Once<RwLock<FsMap>> = Once::new();

pub fn filesystems() -> RwLockReadGuard<'static, FsMap> {
    FILESYSTEM_MAP
        .call_once(|| RwLock::new(BTreeMap::new()))
        .read()
}

pub fn filesystems_mut() -> RwLockWriteGuard<'static, FsMap> {
    FILESYSTEM_MAP
        .call_once(|| RwLock::new(BTreeMap::new()))
        .write()
}

pub fn unmount(id: u64) -> Result<(), ::common::error::Error> {
    match filesystems_mut().get_mut(&id) {
        None => return Err(err!(ENOENT)),
        Some(fs) => {
            try!(fs.unmount());
        }
    };
    match filesystems_mut().remove(&id) {
        None => panic!("Failed when removing fs from map, race condition?"),
        _ => {}
    };
    Ok(())
}

fn insert<T: FileSystem + Sized + 'static>(fs: T) -> Result<u64, ::common::error::Error> {
    let mut fslist = filesystems_mut();
    for idx in 0..MAX_FS {
        match fslist.get(&idx) {
            Some(_) => {}
            None => {
                return match fslist.insert(idx, Box::new(fs)) {
                    None => Ok(idx),
                    Some(_) => panic!("This should not happen.."),
                }
            }
        }
    }
    Err(err!(EFULL))
}

pub fn init() {
    dfs::init();
}
