use core::fmt;
use core::ops::Drop;
use core::sync::atomic;

static LOGGING_LOCK: atomic::AtomicBool = atomic::ATOMIC_BOOL_INIT;

/// Serial port writer
pub struct SerialWriter(bool, bool);

impl SerialWriter {
    pub fn get(module: &str, lf: bool) -> SerialWriter {
        let mut ret = SerialWriter(!LOGGING_LOCK.swap(true, atomic::Ordering::Acquire), lf);
        {
            use core::fmt::Write;
            let _ = write!(&mut ret, "[{}] ", module);
        }
        ret
    }
}

impl Drop for SerialWriter {
    fn drop(&mut self) {
        if self.0 {
            use core::fmt::Write;
            if self.1 {
                let _ = write!(self, "\n");
            }
            LOGGING_LOCK.store(false, atomic::Ordering::Release);
        }
    }
}

impl fmt::Write for SerialWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        if self.0 {
            unsafe {
                ::arch::puts(s);
            }
        }
        Ok(())
    }
}
