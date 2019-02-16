use core::ops::Drop;
use core::sync::atomic;

extern "C" {
    static mut idt_entry: [IdtEntry; 256];
    static mut idt_handlers: [u64; 256];
    static mut idt_descriptor: IdtDescriptor;

    fn idt_init();
}

pub const IDT_INTERRUPT_16: u8 = 0x6;
pub const IDT_TRAP_16: u8 = 0x7;
pub const IDT_INTERRUPT_64: u8 = 0xE;
pub const IDT_TRAP_64: u8 = 0xF;

#[repr(C, packed)]
struct IdtEntry {
    lowbits: u16,
    selector: u16,
    reserved_0: u8,
    attribute: u8,
    midbits: u16,
    hibits: u32,
    reserved_1: u32,
}

#[repr(C, packed)]
struct IdtDescriptor {
    limit: u16,
    base: u64,
}

/// Set IF flag, enable interrupt
#[inline]
pub unsafe fn sti() {
    asm!("sti");
}

/// Clear IF flag, disable interrupt
#[inline]
pub unsafe fn cli() {
    asm!("cli");
}

/// Load IDT descriptor
#[inline]
unsafe fn lidt(ptr: *const IdtDescriptor) {
    asm!("lidt [rax]" : : "{rax}"(ptr) : : "intel", "volatile");
}

/// Breakpoint
#[inline]
pub unsafe fn int3() {
    asm!("int3");
}

/// Check if interrupt enabled
#[inline]
pub unsafe fn check_int() -> bool {
    let flags: u64;
    asm!(r#"
    pushfq
    pop $0
    "# : "=r" (flags) : : );
    return flags & (1 << 9) != 0;
}

type Handler = Option<fn(u64, u64)>;

static mut INTERRUPT_HANDLERS: [Handler; 256] = [None; 256];

/// interrupt handler dispatcher
#[no_mangle]
pub extern "C" fn int_handler(vector: u64, error_code: u64) {
    println!("vector: {}, error_code: {}", vector, error_code);
    unsafe {
        if let Some(ref handler) = INTERRUPT_HANDLERS[vector as usize] {
            handler(vector, error_code);
        }
    }
}

static IDT_LOCK: atomic::AtomicBool = atomic::ATOMIC_BOOL_INIT;

/// A locked instance of IDT
pub struct IDT(bool);

impl IDT {
    /// Get an IDT instance
    pub fn get() -> Self {
        while !IDT_LOCK.compare_and_swap(false, true, atomic::Ordering::Relaxed) {}
        atomic::fence(atomic::Ordering::Acquire);

        IDT(true)
    }

    pub unsafe fn set_entry(&self, idx: usize, handler: u64, selector: u16, dpl: u8, etype: u8) {
        let entry = &mut idt_entry[idx];

        entry.lowbits = (handler & 0xffff) as u16;
        entry.selector = selector;
        entry.reserved_0 = 0;
        entry.attribute = 0x80 | (dpl << 5) | etype;
        entry.midbits = ((handler >> 16) & 0xffff) as u16;
        entry.hibits = ((handler >> 32) & 0xffffffff) as u32;
        entry.reserved_1 = 0;
    }

    /// Set an entry as trap handler (does not clear IF when called)
    pub fn set_kernel_trap(&self, idx: usize, handler: u64) {
        unsafe {
            self.set_entry(idx, handler, ::arch::gdt::GDT_64_CODE, 0, IDT_TRAP_64);
        }
    }

    /// Set an entry as interrupt handler (clear IF when called)
    pub fn set_kernel_isr(&self, idx: usize, handler: u64) {
        unsafe {
            self.set_entry(idx, handler, ::arch::gdt::GDT_64_CODE, 0, IDT_INTERRUPT_64);
        }
    }

    /// Re-load IDT
    pub fn flush(&self) {
        unsafe {
            lidt(&idt_descriptor as *const IdtDescriptor);
        }
    }

    /// Register an ISR handler
    pub fn register_isr(&self, idx: usize, handler: fn(u64, u64)) -> bool {
        unsafe {
            if let Some(ref _hndlr) = INTERRUPT_HANDLERS[idx] {
                return false;
            }
            INTERRUPT_HANDLERS[idx] = Some(handler);
        }
        true
    }

    /// Unregister an ISR handler
    pub fn unregister_isr(&self, idx: usize) -> bool {
        unsafe {
            if let None = INTERRUPT_HANDLERS[idx] {
                return false;
            }
            INTERRUPT_HANDLERS[idx] = None;
        }
        true
    }
}

impl Drop for IDT {
    fn drop(&mut self) {
        IDT_LOCK.store(false, atomic::Ordering::Release);
    }
}

/// Initialize IDT
pub fn init() {
    let idt = IDT::get();

    for i in 0..256 {
        if i == 3 {
            unsafe {
                idt.set_entry(
                    i,
                    idt_handlers[i],
                    ::arch::gdt::GDT_64_CODE,
                    3,
                    IDT_INTERRUPT_64,
                );
            }
        } else {
            unsafe {
                idt.set_entry(
                    i,
                    idt_handlers[i],
                    ::arch::gdt::GDT_64_CODE,
                    0,
                    IDT_INTERRUPT_64,
                );
            }
        }
    }
    idt.flush();
}
