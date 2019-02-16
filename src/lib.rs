#![feature(panic_implementation)]
#![feature(panic_info_message)]
#![feature(asm)]
#![feature(lang_items)]
#![feature(alloc)]
#![feature(alloc_error_handler)]
#![feature(rust_2018_preview)]
#![feature(naked_functions)]
#![no_std]
#![allow(dead_code, non_camel_case_types)]

#[macro_use]
extern crate alloc;
extern crate linked_list_allocator;
extern crate num;
extern crate rlibc;
extern crate spin;

#[macro_use]
mod macros;
mod common;
mod debug;
mod dev;
mod fs;
mod panic;
mod task;

#[cfg(target_arch = "x86_64")]
#[path = "arch/x86_64/mod.rs"]
pub mod arch;

use core::ops::DerefMut;
use linked_list_allocator::LockedHeap;
use task::Task;

#[panic_implementation]
#[no_mangle]
pub fn panic_implementation(info: &::core::panic::PanicInfo) -> ! {
    panic::panic_handler(info);
    loop {}
}

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

#[alloc_error_handler]
#[no_mangle]
pub fn alloc_failure(_: ::core::alloc::Layout) -> ! {
    panic!("Out-of-memory");
}

#[cfg(test)]
pub unsafe fn test() {
    let result = arch::getb();
    println!("Get: {}", result);

    let mut ctx1: *mut Task = 0 as *mut Task;
    let mut ctx2: *mut Task = 0 as *mut Task;
    {
        let mut tasks = task::tasks_mut();
        {
            let task_lock = tasks.new_task().unwrap();
            let mut task = task_lock.write();
            let address = task.context.map(0).expect("Failed to map") as *mut u8;
            task.context.write(address, 0x6a);
            task.context.write(address.offset(1), 0x02);
            task.context.write(address.offset(2), 0xeb);
            task.context.write(address.offset(3), 0xfe);
            task.context.rip = 0x1000;
            ctx1 = task.deref_mut() as *mut Task;
        }

        {
            let task2_lock = tasks.new_task().unwrap();
            let mut task2 = task2_lock.write();
            let address2 = task2.context.map(0).expect("Failed to map address") as *mut u8;
            task2.context.write(address2, 0xeb);
            task2.context.write(address2.offset(1), 0xfe);
            task2.context.rip = 0x1000;
            ctx2 = task2.deref_mut() as *mut Task;
        }

        task::set_current_tid((*ctx1).tid());
        (*ctx1).status = task::TaskStatus::Running;
        (*ctx2).status = task::TaskStatus::Ready;
    }
    (*ctx1).context.switch_to();

    //arch::enable_int();
}

#[no_mangle]
pub extern "C" fn kentry() {
    println!("Hello!");

    // Initialize architecture-dependent features
    arch::init();
    // Initialize heap
    unsafe {
        ALLOCATOR
            .lock()
            .init(arch::HEAP_VIRT as usize, arch::HEAP_SIZE as usize);
    }
    // Second phase architecture-dependent initialization
    arch::init2();
    // Initialize task scheduler
    task::init();

    loop {}
}
