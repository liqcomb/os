use spin::RwLock;

extern "C" {
    static mut init_stack_end: [u64; 1];
}

#[repr(C, packed)]
pub struct TSS {
    reserved_1: u32,
    /// The full 64-bit canonical forms of the stack pointers (RSP) for privilege levels 0-2.
    pub privilege_stack_table: [u64; 3],
    reserved_2: u64,
    /// The full 64-bit canonical forms of the interrupt stack table (IST) pointers.
    pub interrupt_stack_table: [u64; 7],
    reserved_3: u64,
    reserved_4: u16,
    /// The 16-bit offset to the I/O permission bit map from the 64-bit TSS base.
    pub iomap_base: u16,
}

pub static TSS_SC: RwLock<TSS> = RwLock::new(TSS {
    reserved_1: 0,
    reserved_2: 0,
    reserved_3: 0,
    reserved_4: 0,
    privilege_stack_table: [0; 3],
    interrupt_stack_table: [0; 7],
    iomap_base: 0,
});

/// Load TR register
pub unsafe fn ltr(seg: u16) {
    asm!("ltr $0" : : "r"(seg) : : );
}

impl TSS {
    pub fn new() -> Self {
        TSS {
            reserved_1: 0,
            privilege_stack_table: [0; 3],
            reserved_2: 0,
            interrupt_stack_table: [0; 7],
            reserved_3: 0,
            reserved_4: 0,
            iomap_base: 0,
        }
    }
}

pub fn init() {
    let mut tss = TSS_SC.write();
    unsafe {
        tss.privilege_stack_table[0] = &init_stack_end as *const _ as u64;
        tss.interrupt_stack_table[0] = &init_stack_end as *const _ as u64;
        ltr(super::gdt::GDT_TSS);
    }
}
