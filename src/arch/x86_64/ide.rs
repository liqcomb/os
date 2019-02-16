use arch::io;
use common::consts::*;
use core::marker::{Send, Sync};
use dev::{devices_mut, Device};

pub struct IDEDevice {
    diskno: u8,
    offset: u64,
}

unsafe impl Sync for IDEDevice {}
unsafe impl Send for IDEDevice {}

const ATA_SR_BUSY: u8 = 0x80;
const ATA_SR_DRDY: u8 = 0x40;
const ATA_SR_DF: u8 = 0x20;
const ATA_SR_DSC: u8 = 0x10;
const ATA_SR_DRQ: u8 = 0x8;
const ATA_SR_CORR: u8 = 0x4;
const ATA_SR_IDX: u8 = 0x2;
const ATA_SR_ERR: u8 = 0x1;

const SECTOR_SIZE: usize = 512;

/// Switch to diskno and wait
unsafe fn wait_disk() -> bool {
    let mut r: u8;
    loop {
        r = io::inb(0x1F7);
        if r & (ATA_SR_BUSY | ATA_SR_DRDY) == ATA_SR_DRDY {
            break;
        }
    }
    match r & (ATA_SR_DF | ATA_SR_ERR) {
        0 => true,
        _ => false,
    }
}

// MIT 6.828
unsafe fn probe_disk(no: u8) -> bool {
    let mut x: u32 = 0;
    io::outb(0x1F6, 0xE0 | (no << 4));
    loop {
        if x >= 1000 || io::inb(0x1F7) & (ATA_SR_BUSY | ATA_SR_ERR | ATA_SR_DF) == 0 {
            break;
        }
        x += 1;
    }
    return x < 1000;
}

impl IDEDevice {
    pub fn new(diskno: u8) -> Self {
        unsafe {
            probe_disk(diskno);
        }
        Self {
            diskno: diskno,
            offset: 0,
        }
    }
}

/// Read from IDE disk
unsafe fn ide_read(diskno: u8, secno: u32, data: &mut [u8], nsecs: u8) -> bool {
    assert!(data.len() >= (nsecs as usize) * SECTOR_SIZE);

    wait_disk();
    io::outb(0x1F2, nsecs);
    io::outb(0x1F3, (secno & 0xFF) as u8);
    io::outb(0x1F4, ((secno >> 8) & 0xFF) as u8);
    io::outb(0x1F5, ((secno >> 16) & 0xFF) as u8);
    io::outb(
        0x1F6,
        0xE0 | ((diskno & 1) << 4) | (((secno >> 24) & 0xF) as u8),
    );
    io::outb(0x1F7, 0x20);

    let mut ptr = data.as_mut_ptr();
    for _i in 0..nsecs {
        if wait_disk() == false {
            return false;
        }
        io::insl(0x1F0, ptr as *mut u32, (SECTOR_SIZE / 4) as u64);
        ptr = ptr.offset(SECTOR_SIZE as isize);
    }
    true
}

/// Write IDE disk
unsafe fn ide_write(diskno: u8, secno: u32, data: &[u8], nsecs: u8) -> bool {
    assert!(data.len() >= (nsecs as usize) * SECTOR_SIZE);

    wait_disk();
    io::outb(0x1F2, nsecs);
    io::outb(0x1F3, (secno & 0xFF) as u8);
    io::outb(0x1F4, ((secno >> 8) & 0xFF) as u8);
    io::outb(0x1F5, ((secno >> 16) & 0xFF) as u8);
    io::outb(
        0x1F6,
        0xE0 | ((diskno & 1) << 4) | (((secno >> 24) & 0xF) as u8),
    );
    io::outb(0x1F7, 0x30);

    let mut ptr = data.as_ptr();
    for _i in 0..nsecs {
        if wait_disk() == false {
            return false;
        }
        io::outsl(0x1F0, ptr as *const u32, (SECTOR_SIZE / 4) as u64);
        ptr = ptr.offset(SECTOR_SIZE as isize);
    }
    true
}

impl Device for IDEDevice {
    fn read(&mut self, data: &mut [u8]) -> Result<usize, ::common::error::Error> {
        let mut sector: [u8; 1024] = [0; 1024];
        let mut secno: u32 = (self.offset / SECTOR_SIZE as u64) as u32;
        let mut offset: usize = (self.offset as usize) % SECTOR_SIZE;
        let mut remaining = data.len();
        let mut index: usize = 0;

        while remaining > 0 {
            if !unsafe { ide_read(self.diskno, secno, &mut sector, 1) } {
                return Err(err!(EIO));
            }
            for idx in offset..SECTOR_SIZE {
                if remaining == 0 {
                    break;
                }
                data[index] = sector[idx];
                index += 1;
                remaining -= 1;
            }
            secno += 1;
            offset = 0;
        }
        self.offset += data.len() as u64;
        Ok(data.len())
    }

    fn write(&mut self, data: &[u8]) -> Result<usize, ::common::error::Error> {
        let mut sector: [u8; 1024] = [0; 1024];
        let mut secno: u32 = (self.offset / SECTOR_SIZE as u64) as u32;
        let mut offset: usize = (self.offset as usize) % SECTOR_SIZE;
        let mut remaining = data.len();
        let mut index: usize = 0;

        while remaining > 0 {
            if offset != 0 || remaining < SECTOR_SIZE {
                if !unsafe { ide_read(self.diskno, secno, &mut sector, 1) } {
                    return Err(err!(EIO));
                }

                for idx in offset..SECTOR_SIZE {
                    if remaining == 0 {
                        break;
                    }
                    sector[idx] = data[index];
                    index += 1;
                    remaining -= 1;
                }

                if !unsafe { ide_write(self.diskno, secno, &sector, 1) } {
                    return Err(err!(EIO));
                }
                offset = 0;
            } else {
                if !unsafe { ide_write(self.diskno, secno, &data[index..], 1) } {
                    return Err(err!(EIO));
                }
                index += SECTOR_SIZE;
                remaining -= SECTOR_SIZE;
            }
            secno += 1;
        }
        self.offset += data.len() as u64;
        Ok(data.len())
    }

    fn ioctl(&mut self, _ops: u64, _data: usize) -> Result<usize, ::common::error::Error> {
        Err(err!(EFAIL))
    }

    fn seek(&mut self, whence: u32, offset: u64) -> Result<u64, ::common::error::Error> {
        match whence {
            SEEK_SET => {
                self.offset = offset;
                Ok(self.offset)
            }
            SEEK_CUR => {
                self.offset += offset;
                Ok(self.offset)
            }
            _ => Err(err!(EFAIL)),
        }
    }
}

#[cfg(test)]
pub unsafe fn test() {
    use common::utility::hexdump;
    /*
    if !probe_disk(1) {
        panic!("Disk 1 failed");
    }

    let mut buffer = [1u8; 512];
    if !ide_write(1, 0, &mut buffer, 1) {
        println!("Failed read sector");
    } else {
        hexdump(&buffer);
    }
    */

    let mut device = IDEDevice::new(1);
    let mut r1 = [0u8; 5];
    device.read(&mut r1).unwrap();
    hexdump(&r1);
    /*
    let mut r2 = [0u8; 1024];
    device.read(&mut r2).unwrap();
    hexdump(&r2);
    */
    let mut r2 = [5u8; 1030];
    device.write(&mut r2).unwrap();
}

/// Initialization
pub fn init() {
    use alloc::prelude::*;
    let mut devlist = devices_mut();
    devlist.insert("ide0".to_string(), Box::new(IDEDevice::new(0)));
    devlist.insert("ide1".to_string(), Box::new(IDEDevice::new(1)));
}
