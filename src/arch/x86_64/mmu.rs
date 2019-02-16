//! Currently we use only 64Mb physical memory
//! Physical memory layout looks like:
//! [1] [0x0, 0x600000): Initially mapped space, Kernel starts at 0x100000
//! [2] [0x600000, 0x800000): Contiguous kernel memory, contains page table
//!
//! Virtual memory layout looks like:
//! [1] [0xFFFFFFFF80000000, 0xFFFFFFFF80600000)
//! [2] [0xFFFFFFFF80600000, 0xFFFFFFFF80800000)
//!
//! We don't really need to modify PDPT entries
//! after initialization, so we just create the instance
//! of several page tables and two page directories

use core::convert::{From, Into};
use core::ops::Drop;
use core::sync::atomic;
use rlibc::memset;

const PAGETABLE_PHYS: u64 = 0x600000;
const PAGETABLE_VIRT: u64 = 0xFFFFFFFF80600000;
const PAGE_SHIFT: u32 = 12;
const INITIAL_MAPPED: u64 = 3072;
const MAX_MAPPED: u64 = 16384; // 64Mb

extern "C" {
    static mut pml4: [u64; 512];
    static mut user_pdpt: PageTable;
    static mut kernel_pdpt: PageTable;
    static mut user_pd: PageTable;
    static mut kernel_pd: PageTable;
}

/// Structure representing a page table structure (PML4, PDPT, PD, PT)
///
#[repr(C, packed)]
pub struct PageTable {
    v: [u64; 512],
}

impl PageTable {
    /// Generic mapping function
    pub fn map(
        &mut self,
        idx: usize,
        paddr: PhysicalAddress,
        rw: bool,
        user: bool,
        ps: bool,
    ) -> bool {
        // Don't override existing mapping
        if self.v[idx] & 1 != 0 {
            return false;
        }

        let mut entry: u64 = 0x1;
        if ps {
            entry |= 0x80;
        }
        if rw {
            entry |= 0x2;
        }
        if user {
            entry |= 0x4;
        }
        entry |= paddr.mask(12);

        self.v[idx] = entry;
        true
    }

    /// Unmap an entry
    pub fn unmap(&mut self, idx: usize) -> bool {
        if self.v[idx] & 1 != 0 {
            self.v[idx] = 0;
            true
        } else {
            false
        }
    }

    /// Get next level page table
    pub fn next(&self, idx: usize) -> Result<*mut PageTable, ::common::error::Error> {
        // Only when the entry is present and PS flag is not set
        // it's a valid entry for next level
        if self.v[idx] & 1 == 0 || self.v[idx] & 0x80 != 0 {
            return Err(err!(EFAULT));
        }
        // Page table resides in kernel space
        // and it's identically mapped
        let virtaddr = VirtualAddress((self.v[idx] & !((1 << 12) - 1)) + KERNEL_BASE);
        Ok(virtaddr.as_ptr())
    }

    /// Map kernel space
    pub fn map_kernel(&mut self) -> bool {
        let paddr = unsafe { (&kernel_pdpt as *const PageTable) as u64 - KERNEL_BASE };
        self.map(511, paddr.into(), true, false, false)
    }

    /// Check if entry present
    pub fn present(&self, idx: usize) -> bool {
        self.v[idx] & 1 != 0
    }

    /// Get an entry directly
    pub fn get(&self, idx: usize) -> u64 {
        self.v[idx]
    }
}

/// Virtual address
#[derive(Clone, Copy, Debug)]
pub struct VirtualAddress(u64);
/// Physical address
#[derive(Clone, Copy, Debug)]
pub struct PhysicalAddress(u64);

impl VirtualAddress {
    pub fn new(v: u64) -> Self {
        VirtualAddress(v)
    }

    /// Create a VirtualAddress from pointer
    pub fn from_pointer<T>(ptr: *const T) -> Self {
        let value = ptr as u64;
        VirtualAddress(value)
    }

    /// Return page frame number
    #[inline]
    pub fn frame(&self) -> u64 {
        self.0 >> PAGE_SHIFT
    }

    /// Return page table index
    #[inline]
    pub fn table_index(&self) -> usize {
        ((self.0 >> PAGE_SHIFT) & 0x1ff) as usize
    }

    /// Return page directory index
    #[inline]
    pub fn dir_index(&self) -> usize {
        ((self.0 >> 21) & 0x1ff) as usize
    }

    /// Return page directory pointer index
    #[inline]
    pub fn dptr_index(&self) -> usize {
        ((self.0 >> 30) & 0x1ff) as usize
    }

    /// Return PML4 index
    #[inline]
    pub fn pml4_index(&self) -> usize {
        ((self.0 >> 39) & 0x1ff) as usize
    }

    /// Mask low bits
    #[inline]
    pub fn mask(&self, lowbits: u64) -> u64 {
        self.0 & !((1 << lowbits) - 1)
    }

    /// Return if the paddr is an usermode address
    pub fn usermode(&self) -> bool {
        self.pml4_index() == 0
    }

    /// Returns as a pointer
    pub fn as_ptr<T: Sized>(&self) -> *mut T {
        self.0 as *mut T
    }

    /// Returns as a reference
    pub unsafe fn as_ref<T: Sized>(&self) -> Option<&mut T> {
        (self.as_ptr() as *mut T).as_mut()
    }

    /// Subtract and return the result
    pub fn sub(&self, v: u64) -> u64 {
        self.0 - v
    }

    /// Add and return the result
    pub fn add(&self, v: u64) -> u64 {
        self.0 + v
    }
}

impl PhysicalAddress {
    pub fn new(v: u64) -> Self {
        PhysicalAddress(v)
    }

    /// Mask low bits
    #[inline]
    pub fn mask(&self, lowbits: u64) -> u64 {
        self.0 & !((1 << lowbits) - 1)
    }

    /// From Page Frame Number
    #[inline]
    pub fn from_pfn(frame: u64) -> Self {
        PhysicalAddress(frame * PAGE_SIZE)
    }

    /// To PFN
    #[inline]
    pub fn pfn(&self) -> u64 {
        self.0 >> PAGE_SHIFT
    }

    /// Subtract and return the result
    pub fn sub(&self, v: u64) -> u64 {
        self.0 - v
    }

    /// Add and return the result
    pub fn add(&self, v: u64) -> u64 {
        self.0 + v
    }
}

impl From<u64> for PhysicalAddress {
    fn from(v: u64) -> Self {
        PhysicalAddress(v)
    }
}

impl From<u64> for VirtualAddress {
    fn from(v: u64) -> Self {
        VirtualAddress(v)
    }
}

impl Into<u64> for PhysicalAddress {
    fn into(self) -> u64 {
        self.0
    }
}

impl Into<u64> for VirtualAddress {
    fn into(self) -> u64 {
        self.0
    }
}

/// Mapped page size
pub const PAGE_SIZE: u64 = 0x1000;
pub const KERNEL_BASE: u64 = 0xFFFFFFFF80000000;
pub const HEAP_VIRT: u64 = 0xFFFFFFFF80700000;
pub const HEAP_SIZE: u64 = 0x100000;

/// Physical page usage
static mut PHYSPAGE_BITMAP: [u64; 1024] = [0; 1024];
/// Page table usage
static mut PAGETABLE_INUSE: [bool; 1024] = [false; 1024];

/// Marks a 4K page as present
fn mark_page(frame: u64) {
    let idx = frame as usize / 64;
    let bitoff = frame % 64;
    unsafe {
        PHYSPAGE_BITMAP[idx] |= 1 << bitoff;
    }
}

/// Marks a 4K page as free
fn clear_page(frame: u64) {
    let idx = frame as usize / 64;
    let bitoff = frame % 64;
    unsafe {
        PHYSPAGE_BITMAP[idx] &= !(1 << bitoff);
    }
}

/// Checks whether a page is present
fn page_marked(frame: u64) -> bool {
    let idx = frame as usize / 64;
    let bitoff = frame % 64;
    unsafe { (PHYSPAGE_BITMAP[idx] & 1 << bitoff) != 0 }
}

/// Get cr3
#[inline]
pub unsafe fn cr3() -> u64 {
    let result: u64;
    asm!("mov %cr3, $0" : "=r"(result) : : );
    result
}

/// Set cr3
#[inline]
pub unsafe fn set_cr3(cr3: u64) {
    asm!("mov $0, %cr3" : : "r"(cr3) : : );
}

static MMU_LOCK: atomic::AtomicBool = atomic::ATOMIC_BOOL_INIT;

/// An instance of MMU
pub struct MMU(bool);

impl MMU {
    /// Get an instance of MMU
    pub fn get() -> Self {
        while !MMU_LOCK.compare_and_swap(false, true, atomic::Ordering::Relaxed) {
            // Do nothing
        }
        atomic::fence(atomic::Ordering::Acquire);

        MMU(true)
    }

    /// Virtual address to physical address
    pub fn vtop(addr: VirtualAddress) -> Result<PhysicalAddress, ::common::error::Error> {
        unimplemented!();
    }

    /// Flush entire TLB
    pub unsafe fn flush(&self) {
        asm!(
            r#"
        mov %rax, %cr3
        mov %cr3, %rax
        "#
        );
    }

    /// Allocate one physical page
    pub fn alloc_phys(&self) -> Result<PhysicalAddress, ::common::error::Error> {
        for frame in INITIAL_MAPPED..MAX_MAPPED {
            if !page_marked(frame) {
                mark_page(frame);
                return Ok(PhysicalAddress::from_pfn(frame));
            }
        }
        Err(err!(ENOMEM))
    }

    /// Free one physical page
    pub fn free_phys(&self, addr: PhysicalAddress) -> Result<(), ::common::error::Error> {
        let frame = addr.pfn();
        match page_marked(frame) {
            false => Err(err!(EFAULT)),
            true => {
                clear_page(frame);
                Ok(())
            }
        }
    }

    /// Allocate one page
    pub fn alloc_page(&self) -> Result<VirtualAddress, ::common::error::Error> {
        // Page table region: 0x600000 ~ 0x700000
        for i in 0..1024 {
            unsafe {
                if !PAGETABLE_INUSE[i] {
                    PAGETABLE_INUSE[i] = true;
                    let address: u64 = (i as u64) * PAGE_SIZE + PAGETABLE_VIRT;
                    memset(address as *mut u8, 0, PAGE_SIZE as usize);
                    return Ok(address.into());
                }
            }
        }
        Err(err!(ENOMEM))
    }

    /// Free one page
    pub fn free_page(&self, virt: VirtualAddress) -> Result<(), ::common::error::Error> {
        let address: u64 = (Into::<u64>::into(virt) - PAGETABLE_VIRT) / PAGE_SIZE;
        unsafe {
            if PAGETABLE_INUSE[address as usize] {
                PAGETABLE_INUSE[address as usize] = false;
                return Ok(());
            }
        }
        Err(err!(EFAULT))
    }

    /// Allocate contiguous pages
    pub fn alloc_contiguous(&self, count: usize) -> Result<VirtualAddress, ::common::error::Error> {
        for i in 0..1024 {
            // Look for contiguous pages linearly
            let mut flag = true;
            for j in i..i + count {
                unsafe {
                    if j >= 1024 || PAGETABLE_INUSE[j] {
                        flag = false;
                        break;
                    }
                }
            }

            // Returns if ok
            if flag {
                for j in i..i + count {
                    unsafe {
                        PAGETABLE_INUSE[j] = true;
                    }
                }
                let addr = (i as u64) * PAGE_SIZE + PAGETABLE_VIRT;
                return Ok(addr.into());
            }
        }
        Err(err!(ENOMEM))
    }

    /// Free contiguous pages
    pub fn free_contiguous(
        &self,
        addr: VirtualAddress,
        count: usize,
    ) -> Result<(), ::common::error::Error> {
        for i in 0..count {
            let address = addr.mask(12) + (i as u64) * PAGE_SIZE;
            try!(self.free_page(address.into()));
        }
        Ok(())
    }

    /// Return current PML4 Virtual address
    pub fn pml4(&self) -> *mut PageTable {
        unsafe {
            let virtaddr = VirtualAddress(cr3() + KERNEL_BASE);
            virtaddr.as_ptr()
        }
    }

    /// Create a initialized page table for a new context
    pub fn new_pml4(&self) -> Result<*mut PageTable, ::common::error::Error> {
        unimplemented!();
    }

    /// Free a created PML4
    pub fn free_pml4(&self) {
        unimplemented!();
    }
}

impl Drop for MMU {
    fn drop(&mut self) {
        MMU_LOCK.store(false, atomic::Ordering::Release);
    }
}

pub fn init() {
    // These pages are mapped initially
    for i in 0..INITIAL_MAPPED {
        mark_page(i);
    }

    unsafe {
        // Replace the first pml4 entry
        let ptr = ((&mut pml4 as *mut [u64; 512]) as u64 + KERNEL_BASE) as *mut u64;
        let val = (&mut user_pdpt as *mut PageTable) as u64 - KERNEL_BASE;
        *ptr = val;

        // Map contiguous kernel memory
        let addr = 0x600000;
        assert!(kernel_pd.map(3, addr.into(), true, false, true));
    }
}
