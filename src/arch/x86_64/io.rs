/// Write a byte to the specified port
pub unsafe fn outb(port: u16, val: u8) {
    asm!("outb $0, $1" : : "{al}"(val), "{dx}N"(port));
}

/// Read a single byte from the specified port
pub unsafe fn inb(port: u16) -> u8 {
    let ret: u8;
    asm!("inb $1, $0" : "={al}"(ret) : "{dx}N"(port));
    return ret;
}

/// Write a word (16-bits) to the specified port
pub unsafe fn outw(port: u16, val: u16) {
    asm!("outw $0, $1" : : "{ax}"(val), "{dx}N"(port));
}

/// Read a word (16-bits) from the specified port
pub unsafe fn inw(port: u16) -> u16 {
    let ret: u16;
    asm!("inw $1, $0" : "={ax}"(ret) : "{dx}N"(port));
    return ret;
}

/// Write a dword to the specified port
pub unsafe fn outl(port: u16, val: u32) {
    asm!("outl $0, $1" : : "{eax}"(val), "{dx}N"(port));
}

/// Read a dword from the specified port
pub unsafe fn inl(port: u16) -> u32 {
    let ret: u32;
    asm!("inl $1, $0" : "={ax}"(ret) : "{dx}N"(port));
    return ret;
}

/// Read string from port
pub unsafe fn insl(port: u16, data: *mut u32, cnt: u64) {
    asm!("repne insl" : : "{rdi}"(data), "{dx}N"(port), "{rcx}"(cnt));
}

pub unsafe fn outsl(port: u16, data: *const u32, cnt: u64) {
    asm!("repne outsl" : : "{rsi}"(data), "{dx}N"(port), "{rcx}"(cnt));
}
