mod context;
mod gdt;
mod ide;
mod idt;
mod io;
mod mmu;
mod pci;
mod pic;
mod timer;
mod tss;

/* exposed child definitions */
pub use self::context::Context;

use self::idt::IDT;
use self::timer::Timer;

pub const HEAP_VIRT: u64 = mmu::HEAP_VIRT;
pub const HEAP_SIZE: u64 = mmu::HEAP_SIZE;

/* exported symbols */
pub use self::context::store_context;
pub use self::idt::int_handler;

/// Put a string of bytes to serial port
pub unsafe fn puts(s: &str) {
    for b in s.bytes() {
        putb(b);
    }
}

/// Put a byte to serial port
pub unsafe fn putb(b: u8) {
    // Wait for the serial port's fifo to not be empty
    while (io::inb(0x3F8 + 5) & 0x20) == 0 {
        // Do nothing
    }
    // Send the byte out the serial port
    io::outb(0x3F8, b);
}

/// Get a byte from serial port
pub unsafe fn getb() -> u8 {
    while (io::inb(0x3F8 + 5) & 0x1) == 0 {
        // Do nothing
    }
    io::inb(0x3F8)
}

/// Disable interrupt
pub unsafe fn disable_int() {
    idt::cli();
}

/// Enable interrupt
pub unsafe fn enable_int() {
    idt::sti();
}

/// Check whether interrupt is enabled
pub unsafe fn int_enabled() -> bool {
    idt::check_int()
}

/// Breakpoint
pub unsafe fn breakpoint() {
    idt::int3();
}

/// Register an ISR handler
pub fn register_isr(idx: usize, handler: fn(u64, u64)) -> bool {
    IDT::get().register_isr(idx, handler)
}

/// Unregister an ISR handler
pub fn unregister_isr(idx: usize) -> bool {
    IDT::get().unregister_isr(idx)
}

/// Register a timer handler
pub fn register_timer(func: fn(u64)) -> Result<usize, ::common::error::Error> {
    Timer::get().register_timer(func)
}

/// Unregister a timer handler
pub fn unregister_timer(idx: usize) -> Result<(), ::common::error::Error> {
    Timer::get().unregister_timer(idx)
}

/// Register a scheduler
pub fn register_scheduler(func: fn(u64)) -> Result<(), ::common::error::Error> {
    Timer::get().register_scheduler(func)
}

/// Initialize architecture-related configuration
pub fn init() {
    gdt::init();
    tss::init();
    idt::init();
    pic::init();
    mmu::init();
    timer::init();
    pci::init();
}

/// Phase 2 initialization
pub fn init2() {
    ide::init();
}

#[cfg(test)]
pub fn test() {
    unsafe {
        ide::test();
    }
}
