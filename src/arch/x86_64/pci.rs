use arch::io;

const CONFIG_ADDRESS: u16 = 0xCF8;
const CONFIG_DATA: u16 = 0xCFC;

/// Read from PCI config space
pub unsafe fn pci_readcfg(bus: u32, dev: u32, func: u32, offset: u32) -> u32 {
    let address = 0x80000000u32 | (bus << 16) | (dev << 11) | (func << 8) | (offset & 0xfc);
    io::outl(CONFIG_ADDRESS, address);
    return io::inl(CONFIG_DATA);
}

/// Write to PCI config space
pub unsafe fn pci_writecfg(bus: u32, dev: u32, func: u32, offset: u32, value: u32) {
    let address = 0x80000000u32 | (bus << 16) | (dev << 11) | (func << 8) | (offset & 0xfc);
    io::outl(CONFIG_ADDRESS, address);
    io::outl(CONFIG_DATA, value);
}

pub struct PCIBus {
    busno: u32,
}

// https://wiki.osdev.org/PCI#The_PCI_Bus
impl PCIBus {
    pub fn new(busno: u32) -> Self {
        Self { busno: busno }
    }

    /// scan the devices on the bus
    pub fn scan<T>(&self, attach: T) -> (usize, usize)
    where
        T: Fn(u32, u32, u32) -> bool,
    {
        let mut devcnt: usize = 0;
        let mut attached: usize = 0;

        for dev in 0..32 {
            let vendor = unsafe { pci_readcfg(self.busno, dev, 0, 0) } & 0xFFFF;
            // non-existing device
            if vendor == 0xFFFF {
                continue;
            }
            devcnt += 1;

            let bhlc = unsafe { pci_readcfg(self.busno, dev, 0, 0xC) };
            let funcs: u32;
            if (bhlc >> 16) & 0xFF > 0x80 {
                funcs = 8;
            } else {
                funcs = 1;
            }

            for func in 0..funcs {
                let vendor = unsafe { pci_readcfg(self.busno, dev, func, 0) } & 0xFFFF;
                if vendor == 0xFFFF {
                    continue;
                }

                if attach(self.busno, dev, func) {
                    attached += 1;
                }
            }
        }

        (devcnt, attached)
    }
}

pub fn init() {
    PCIBus::new(0).scan(|busno, dev, func| {
        println!("Busno {} Dev {} Fn {}", busno, dev, func);
        let data = unsafe { pci_readcfg(busno, dev, func, 0x8) };
        println!("Class {} Subclass {}", data >> 24, (data >> 16) & 0xFF);
        false
    });
}
