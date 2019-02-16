use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::string::String;
use spin::{Once, RwLock, RwLockReadGuard, RwLockWriteGuard};

mod device;

pub use self::device::Device;

pub static DEVICE_MAP: Once<RwLock<BTreeMap<String, Box<device::Device>>>> = Once::new();

pub fn devices() -> RwLockReadGuard<'static, BTreeMap<String, Box<device::Device>>> {
    DEVICE_MAP.call_once(|| RwLock::new(BTreeMap::new())).read()
}

pub fn devices_mut() -> RwLockWriteGuard<'static, BTreeMap<String, Box<device::Device>>> {
    DEVICE_MAP
        .call_once(|| RwLock::new(BTreeMap::new()))
        .write()
}
