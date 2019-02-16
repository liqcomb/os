use arch::gdt;
use arch::mmu::{PageTable, PhysicalAddress, VirtualAddress, KERNEL_BASE, MMU, PAGE_SIZE};
use core::ops::Drop;
use task::tasks;

/// General purpose registers
#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
pub struct GPR {
    rax: u64,
    rcx: u64,
    rdx: u64,
    rbx: u64,
    rsi: u64,
    rdi: u64,
    r8: u64,
    r9: u64,
    r10: u64,
    r11: u64,
    r12: u64,
    r13: u64,
    r14: u64,
    r15: u64,
}

/// Segment registers
/// We use 64bit unsigned numbers here for convenience
#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
pub struct SR {
    cs: u64,
    ds: u64,
    es: u64,
    fs: u64,
    gs: u64,
    ss: u64,
}

/// Structure representing architecture-dependent context
#[repr(C, packed)]
pub struct Context {
    pub rflags: u64,
    pub cr3: u64,
    pub rsp: u64,
    pub rip: u64,
    pub rbp: u64,

    page_table: VirtualAddress,
    kernel_stack: VirtualAddress,

    pub gpr: GPR,
    pub sr: SR,
}

impl Context {
    /// Create a new context
    pub fn new() -> Self {
        // Starts at usermode
        let user_cs: u64 = (gdt::GDT_64_USER_CODE | 3).into();
        let user_ds: u64 = (gdt::GDT_64_USER_DATA | 3).into();
        let user_ss: u64 = (gdt::GDT_64_USER_DATA | 3).into();
        let sr = SR {
            cs: user_cs,
            ds: user_ds,
            es: user_ds,
            fs: user_ds,
            gs: user_ds,
            ss: user_ss,
        };

        let gpr = GPR {
            rax: 0,
            rcx: 0,
            rdx: 0,
            rbx: 0,
            rsi: 0,
            rdi: 0,
            r8: 0,
            r9: 0,
            r10: 0,
            r11: 0,
            r12: 0,
            r13: 0,
            r14: 0,
            r15: 0,
        };

        // Creates a page table for context
        // Currently we use only one page table in page directory
        // Since we only have 64Mb, one page table could essentially hold
        // 2Mb, which is more than enough for this environment.
        let mmu = MMU::get();
        let pml4_vaddr = mmu.alloc_page().expect("Failed to allocate PML4");
        let pdpt_vaddr = mmu.alloc_page().expect("Failed to allocate PDPT");
        let pd_vaddr = mmu.alloc_page().expect("Failed to allocate PD");
        let pt_vaddr = mmu.alloc_page().expect("Failed to allocate PT");
        unsafe {
            let pml4: *mut PageTable = pml4_vaddr.as_ptr();
            assert!((*pml4).map_kernel());
            assert!((*pml4).map(0, pdpt_vaddr.sub(KERNEL_BASE).into(), true, true, false));
            let pdpt: *mut PageTable = pdpt_vaddr.as_ptr();
            assert!((*pdpt).map(0, pd_vaddr.sub(KERNEL_BASE).into(), true, true, false));
            let pd: *mut PageTable = pd_vaddr.as_ptr();
            assert!((*pd).map(0, pt_vaddr.sub(KERNEL_BASE).into(), true, true, false));
        }
        let cr3 = pml4_vaddr.sub(KERNEL_BASE);

        // Creates kernel stack
        let kernel_stack = mmu
            .alloc_contiguous(4)
            .expect("Failed to allocate kernel stack");

        let mut context = Context {
            rflags: (0 << 12 | 1 << 9), // IOPL & IF
            cr3: cr3,
            rsp: 0,
            rip: 0,
            rbp: 0,

            page_table: pml4_vaddr,
            kernel_stack: kernel_stack,

            gpr: gpr,
            sr: sr,
        };

        // Creates user stack
        let pt: *mut PageTable = pt_vaddr.as_ptr();
        for i in 0..4 {
            let phys = mmu.alloc_phys().expect("Failed to allocate user stack");
            unsafe {
                (*pt).map(511 - i, phys, true, true, false);
            }
        }
        context.rsp = 0x1FF000;

        context
    }

    /// Allocate a physical page
    /// and maps it to the current
    /// task environment
    pub fn map(&self, address: u64) -> Result<u64, ::common::error::Error> {
        let mmu = MMU::get();
        unsafe {
            let pml4: *mut PageTable = self.page_table.as_ptr();
            let pdpt = try!((*pml4).next(0));
            let pd = try!((*pdpt).next(0));
            let pt = try!((*pd).next(0));

            // Find an available index or use specified index
            let vaddr = VirtualAddress::new(address);
            let mut idx: usize = 0;
            if vaddr.table_index() == 0 {
                for i in 1..512 {
                    if !(*pt).present(i) {
                        idx = i;
                        break;
                    }
                }
            } else {
                idx = vaddr.table_index();
            }

            if idx == 0 || (*pt).present(idx) {
                return Err(err!(ENOMEM));
            }

            let phys = try!(mmu.alloc_phys());

            if !(*pt).map(idx, phys, true, true, false) {
                return Err(err!(EFAULT));
            }

            return Ok(idx as u64 * PAGE_SIZE);
        }
    }

    /// Write on behalf of this context
    pub unsafe fn write<T: Sized>(&self, ptr: *mut T, val: T) {
        let mut saved_cr3: u64 = 0xFFF;
        if super::mmu::cr3() != self.cr3 {
            saved_cr3 = super::mmu::cr3();
            super::mmu::set_cr3(self.cr3);
        }

        (*ptr) = val;

        if saved_cr3 != 0xFFF {
            super::mmu::set_cr3(saved_cr3);
        }
    }

    /// Read on behalf of this context
    pub unsafe fn read<T: Sized + Copy>(&self, ptr: *mut T) -> T {
        let mut saved_cr3: u64 = 0xFFF;
        if super::mmu::cr3() != self.cr3 {
            saved_cr3 = super::mmu::cr3();
            super::mmu::set_cr3(self.cr3);
        }

        let val = *ptr;

        if saved_cr3 != 0xFFF {
            super::mmu::set_cr3(saved_cr3);
        }

        val
    }

    /// Switch to this context
    /// This should only be called in kernel mode
    #[naked]
    #[inline(never)]
    pub unsafe fn switch_to(&self) -> ! {
        asm!(r#"
        /* push interrupt stack frame */
        push qword ptr [rbx+0x28]   /* ss */ 
        push qword ptr [rcx+0x10]   /* rsp */ 
        push qword ptr [rcx+0x0]    /* rflags */ 
        push qword ptr [rbx+0x0]    /* cs */ 
        push qword ptr [rcx+0x18]   /* rip */

        /* switch page table */
        mov rdx, [rcx+0x8]
        mov rsi, cr3
        cmp rdx, rsi
        je skip_cr3
        mov cr3, rdx
    skip_cr3:

        /* set gpr & sr */
        mov rsi, [rax+0x20]
        mov rdi, [rax+0x28]
        mov r8, [rax+0x30]
        mov r9, [rax+0x38]
        mov r10, [rax+0x40]
        mov r11, [rax+0x48]
        mov r12, [rax+0x50]
        mov r13, [rax+0x58]
        mov r14, [rax+0x60]
        mov r15, [rax+0x68]
        mov rdx, [rbx+0x8]
        mov ds, dx
        mov rdx, [rbx+0x10]
        mov es, dx
        mov rdx, [rbx+0x18]
        mov fs, dx
        mov rdx, [rbx+0x20]
        mov gs, dx 
        mov rbp, [rcx+0x20]
        mov rcx, [rax+0x8]
        mov rdx, [rax+0x10]
        mov rbx, [rax+0x18]
        push [rax]
        pop rax 

        /* go back to work */
        iretq 
        "#
        :
        :
        "{rax}" (&self.gpr as *const GPR),
        "{rbx}" (&self.sr as *const SR),
        "{rcx}" (self as *const Context)
        :   // No clobbers
        :
        "intel", "volatile" 
            );
        unreachable!();
    }
}

// drop(context) should be called in dispatcher
// when using then stack of other tasks
// in that case we won't crash for cleaning
// the environment which is currently in use.
impl Drop for Context {
    fn drop(&mut self) {
        let mmu = MMU::get();

        // Free kernel stack
        mmu.free_contiguous(self.kernel_stack, 4)
            .expect("Failed to free kernel stack");

        // Free physical pages allocated
        unsafe {
            let pml4: *mut PageTable = self.page_table.as_ptr();
            let pdpt = (*pml4).next(0).expect("Corrupted page table");
            let pd = (*pdpt).next(0).expect("Corrupted page table");
            let pt = (*pd).next(0).expect("Corrupted page table");
            for i in 0..512 {
                if (*pt).present(i) {
                    let paddr: PhysicalAddress = PhysicalAddress::new((*pt).get(i)).mask(12).into();
                    mmu.free_phys(paddr).expect("Invalid page table entry");
                }
            }
            let pt_vaddr = VirtualAddress::from_pointer(pt);
            mmu.free_page(pt_vaddr)
                .expect("Invalid page directory entry");
            let pd_vaddr = VirtualAddress::from_pointer(pd);
            mmu.free_page(pd_vaddr).expect("Invalid PDPT entry");
            let pdpt_vaddr = VirtualAddress::from_pointer(pdpt);
            mmu.free_page(pdpt_vaddr).expect("Invalid PML4 entry");
            let pml4_vaddr = VirtualAddress::from_pointer(pml4);
            mmu.free_page(pml4_vaddr).expect("Invalid PML4");
        }
    }
}

#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
pub struct CapturedContext {
    cr3: u64,
    rbp: u64,
    gs: u64,
    fs: u64,
    es: u64,
    ds: u64,
    gpr: GPR,
    reserved_0: u64, //  return address
    reserved_1: u64, //  interrupt vector
    reserved_2: u64, //  error code
    rip: u64,
    cs: u64,
    rflags: u64,
    rsp: u64,
    ss: u64,
}

/// This is called by save_context
/// which is called upon interrupt
/// happens.
#[no_mangle]
pub extern "C" fn store_context(ctx: *const CapturedContext) {
    let tasks = tasks();
    {
        let current_lock = tasks.current();
        if !current_lock.is_none() {
            let mut current = current_lock.unwrap().write();
            unsafe {
                current.context.rsp = (*ctx).rsp;
                current.context.rbp = (*ctx).rbp;
                current.context.cr3 = (*ctx).cr3;
                current.context.rip = (*ctx).rip;
                current.context.rflags = (*ctx).rflags;
                current.context.sr.cs = (*ctx).cs;
                current.context.sr.ds = (*ctx).ds;
                current.context.sr.es = (*ctx).es;
                current.context.sr.fs = (*ctx).fs;
                current.context.sr.gs = (*ctx).gs;
                current.context.sr.ss = (*ctx).ss;
                current.context.gpr.rax = (*ctx).gpr.rax;
                current.context.gpr.rcx = (*ctx).gpr.rcx;
                current.context.gpr.rdx = (*ctx).gpr.rdx;
                current.context.gpr.rbx = (*ctx).gpr.rbx;
                current.context.gpr.rsi = (*ctx).gpr.rsi;
                current.context.gpr.rdi = (*ctx).gpr.rdi;
                current.context.gpr.r8 = (*ctx).gpr.r8;
                current.context.gpr.r9 = (*ctx).gpr.r9;
                current.context.gpr.r10 = (*ctx).gpr.r10;
                current.context.gpr.r11 = (*ctx).gpr.r11;
                current.context.gpr.r12 = (*ctx).gpr.r12;
                current.context.gpr.r13 = (*ctx).gpr.r13;
                current.context.gpr.r14 = (*ctx).gpr.r14;
                current.context.gpr.r15 = (*ctx).gpr.r15;
            }
        }
    }
}
