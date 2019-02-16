use super::tss::TSS;
use core::mem::size_of;
use core::ops::{DerefMut, Drop};
use core::sync::atomic;

extern "C" {
    static mut gdt64: GdtDescriptor;
    static mut gdt: [GdtEntry; 10];
}

// Currently available segments
pub const GDT_64_CODE: u16 = 0x8;
pub const GDT_64_DATA: u16 = 0x10;
pub const GDT_32_USER_CODE: u16 = 0x18;
pub const GDT_32_USER_DATA: u16 = 0x20;
pub const GDT_64_USER_CODE: u16 = 0x28;
pub const GDT_64_USER_DATA: u16 = 0x30;
pub const GDT_TSS: u16 = 0x38;

// System segments take two u64
const GDT_TSS_AVAIL: u8 = 0x9;
const GDT_TSS_BUSY: u8 = 0xB;

#[repr(C, packed)]
struct GdtDescriptor {
    limit: u16,
    base: u64,
}

#[repr(C, packed)]
struct GdtEntry {
    low_limit: u16,
    low_base: u16,
    mid_base: u8,
    attribute: u8,
    mixed: u8,
    hi_base: u8,
}

#[inline(never)]
unsafe fn lgdt(ptr: *const GdtDescriptor) {
    asm!("lgdt ($0)" : : "r"(ptr) : "memory");
}

static GDT_LOCK: atomic::AtomicBool = atomic::ATOMIC_BOOL_INIT;

/// An instance of GDT
pub struct GDT(bool);

impl GDT {
    pub fn get() -> Self {
        while !GDT_LOCK.compare_and_swap(false, true, atomic::Ordering::Relaxed) {}
        atomic::fence(atomic::Ordering::Acquire);

        GDT(true)
    }

    pub unsafe fn set_entry(
        &self,
        idx: usize,
        limit: u32,
        base: u32,
        dtype: u8,
        dpl: u8,
        user: bool,
        avl: bool,
        long: bool,
        db: bool,
        g: bool,
    ) {
        let entry = &mut gdt[idx];

        entry.low_limit = (limit & 0xffff) as u16;
        entry.low_base = (base & 0xffff) as u16;
        entry.mid_base = ((base >> 16) & 0xff) as u8;
        entry.attribute = (dtype & 0xf) | ((dpl & 0x3) << 5) | (0x80);
        if user {
            entry.attribute |= 1 << 4;
        }
        entry.mixed = ((limit >> 16) & 0xf) as u8;
        if avl {
            entry.mixed |= 1 << 4;
        }
        if long {
            entry.mixed |= 1 << 5;
        }
        if db {
            entry.mixed |= 1 << 6;
        }
        if g {
            entry.mixed |= 1 << 7;
        }
        entry.hi_base = ((base >> 24) & 0xff) as u8;
    }

    pub unsafe fn set(&self, idx: usize, v: u64) {
        let entry = (&mut gdt[idx] as *mut _ as u64) as *mut u64;

        *entry = v;
    }

    pub fn flush(&self) {
        unsafe {
            lgdt(&gdt64 as *const GdtDescriptor);
        }
    }
}

impl Drop for GDT {
    fn drop(&mut self) {
        GDT_LOCK.store(false, atomic::Ordering::Release);
    }
}

pub fn init() {
    let mut tss = super::tss::TSS_SC.write();
    unsafe {
        let g = GDT::get();
        let limit = size_of::<TSS>() as u32 - 1;
        let base = tss.deref_mut() as *mut TSS as u64;

        g.set_entry(
            7,
            limit,
            (base & 0xFFFFFFFF) as u32,
            GDT_TSS_AVAIL,
            0,
            false,
            false,
            false,
            false,
            false,
        );
        g.set(8, (base >> 32) & 0xFFFFFFFF);
        g.flush();
    }
}
