use arch::io;
use core::ops::Drop;
use core::sync::atomic;

const PIC_ISR_START: u8 = 32;

// Port addresses
const MASTER_PIC: u16 = 0x20;
const MASTER_DATA: u16 = 0x21;
const SLAVE_PIC: u16 = 0xA0;
const SLAVE_DATA: u16 = 0xA1;

const PIC_EOI: u8 = 0x20;

// Control words
const ICW1_ICW4: u8 = 0x1; /* ICW4 (not) needed */
const ICW1_SINGLE: u8 = 0x2; /* Single (cascade) mode */
const ICW1_INTERVAL4: u8 = 0x4; /* Call address interval 4 (8) */
const ICW1_LEVEL: u8 = 0x8; /* Level triggered (edge) mode */
const ICW1_INIT: u8 = 0x10; /* Initialization - required! */

const ICW4_8086: u8 = 0x1; /* 8086/88 (MCS-80/85) mode */
const ICW4_AUTO: u8 = 0x2; /* Auto (normal) EOI */
const ICW4_BUF_SLAVE: u8 = 0x8; /* Buffered mode/slave */
const ICW4_BUF_MASTER: u8 = 0xC; /* Buffered mode/master */
const ICW4_SFNM: u8 = 0x10; /* Special fully nested (not) */

/// An instance of the PIC
pub struct PIC(bool);

static PIC_LOCK: atomic::AtomicBool = atomic::ATOMIC_BOOL_INIT;

impl PIC {
    /// Get a locked instance of local PIC
    pub fn get() -> Self {
        while !PIC_LOCK.compare_and_swap(false, true, atomic::Ordering::Relaxed) {}
        atomic::fence(atomic::Ordering::Acquire);

        PIC(true)
    }

    /// Send EOI command
    pub unsafe fn eoi(slave: bool) {
        if slave {
            io::outb(SLAVE_PIC, PIC_EOI);
        }
        io::outb(MASTER_PIC, PIC_EOI);
    }

    /// Disable 8259 PIC
    pub unsafe fn disable(&self) {
        io::outb(SLAVE_DATA, 0xff);
        io::outb(MASTER_DATA, 0xff);
    }

    /// Mask an interrupt request
    pub unsafe fn mask(&self, idx: u8, slave: bool) {
        let port: u16;
        if slave {
            port = SLAVE_DATA;
        } else {
            port = MASTER_DATA;
        }
        let target = io::inb(port) | (1 << idx);
        io::outb(port, target);
    }

    /// Unmask an interrupt request
    pub unsafe fn unmask(&self, idx: u8, slave: bool) {
        let port: u16;
        if slave {
            port = SLAVE_DATA;
        } else {
            port = MASTER_DATA;
        }
        let target = io::inb(port) & !(1 << idx);
        io::outb(port, target);
    }

    /// Re-maps the interrupt vector
    pub unsafe fn remap(&self, master_offset: u8, slave_offset: u8) {
        let master_mask = io::inb(MASTER_DATA);
        let slave_mask = io::inb(SLAVE_DATA);

        io::outb(MASTER_PIC, ICW1_INIT + ICW1_ICW4);
        io::outb(SLAVE_PIC, ICW1_INIT + ICW1_ICW4);
        io::outb(MASTER_DATA, master_offset);
        io::outb(SLAVE_DATA, slave_offset);
        io::outb(MASTER_DATA, 4); // https://wiki.osdev.org/Interrupts
        io::outb(SLAVE_DATA, 2);
        io::outb(MASTER_DATA, ICW4_8086);
        io::outb(SLAVE_DATA, ICW4_8086);

        // restore mask
        io::outb(MASTER_DATA, master_mask);
        io::outb(SLAVE_DATA, slave_mask);
    }
}

impl Drop for PIC {
    fn drop(&mut self) {
        PIC_LOCK.store(false, atomic::Ordering::Release);
    }
}

pub fn init() {
    unsafe {
        PIC::get().remap(PIC_ISR_START, PIC_ISR_START + 8);
    }
}
