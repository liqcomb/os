use arch::idt::IDT;
use arch::io;
use arch::pic::PIC;
use spin::Mutex;

type TimerCallback = fn(u64);

const MAX_CALLBACKS: usize = 30;
const TIMER_C0_DATA: u16 = 0x40;
const TIMER_C1_DATA: u16 = 0x41;
const TIMER_C2_DATA: u16 = 0x42;
const TIMER_MODE_CTRL: u16 = 0x43;
const TIMER_WRAP: u64 = 0x100000000;

static mut SCHEDULER: Option<TimerCallback> = None;
static TIMER: Mutex<Timer> = Mutex::new(Timer {
    handlers: [None; MAX_CALLBACKS],
    tick: 0,
});

/// Represents an instance of timer handler
pub struct Timer {
    handlers: [Option<TimerCallback>; MAX_CALLBACKS],
    tick: u64,
}

impl Timer {
    /// Get an instance of the timer
    pub fn get() -> spin::MutexGuard<'static, Timer> {
        TIMER.lock()
    }

    /// Register a timer function
    pub fn register_timer(&mut self, func: fn(u64)) -> Result<usize, ::common::error::Error> {
        for i in 0..MAX_CALLBACKS {
            if self.handlers[i].is_some() {
                continue;
            }
            self.handlers[i] = Some(func);
            return Ok(i);
        }
        Err(err!(EFULL))
    }

    /// Unregister a timer function
    pub fn unregister_timer(&mut self, idx: usize) -> Result<(), ::common::error::Error> {
        if self.handlers[idx].is_none() {
            return Err(err!(ENOENT));
        }
        self.handlers[idx] = None;
        Ok(())
    }

    /// Register a scheduler function
    pub fn register_scheduler(&self, func: fn(u64)) -> Result<(), ::common::error::Error> {
        unsafe {
            if SCHEDULER.is_some() {
                return Err(err!(EAGAIN));
            }
            SCHEDULER = Some(func);
        }
        Ok(())
    }
}

fn handler(_vector: u64, _error_code: u64) {
    // Send EOI to Master PIC
    unsafe {
        PIC::eoi(false);
    }

    // Try lock the TIMER global
    // If we cannot get a locked instance of TIMER,
    // we can't just wait here since we're in the
    // middle of an ISR.
    let mut tick: u64 = 0;
    match TIMER.try_lock() {
        None => {
            // Nothing we can do now, TIMER is occupied
        }
        Some(mut timer) => {
            // wrap-adds the tick
            if timer.tick >= TIMER_WRAP {
                timer.tick = 0;
            }
            timer.tick += 1;

            tick = timer.tick;

            for i in 0..MAX_CALLBACKS {
                // Call the callback, the callback must returns
                if let Some(ref func) = timer.handlers[i] {
                    func(timer.tick);
                }
            }
        }
    };

    // Now do the scheduler thing
    // It's totally fine if this does not returns
    unsafe {
        if let Some(ref handler) = SCHEDULER {
            handler(tick);
        }
    }
}

pub fn init() {
    unsafe {
        io::outb(TIMER_MODE_CTRL, 0x36);
        io::outb(TIMER_C0_DATA, 0);
        io::outb(TIMER_C0_DATA, 0);
    }
    assert!(IDT::get().register_isr(32, handler));
}
