use arch::Context;

#[repr(u8)]
#[derive(PartialEq, Eq)]
pub enum TaskStatus {
    Initializing,
    Ready,
    Running,
    Terminated,
}

pub struct Task {
    // task context
    pub context: Context,
    // task ID
    tid: u64,
    // Status
    pub status: TaskStatus,
    // Exit code
    exit_code: u64,
}

impl Task {
    pub fn new(tid: u64) -> Self {
        Task {
            context: Context::new(),
            tid: tid,
            status: TaskStatus::Initializing,
            exit_code: 0,
        }
    }

    /// Check if task is terminated
    pub fn died(&self) -> bool {
        self.status == TaskStatus::Terminated
    }

    /// Check if task is ready to run
    pub fn standby(&self) -> bool {
        self.status == TaskStatus::Ready
    }

    /// Get task exit code
    pub fn exit_code(&self) -> u64 {
        self.exit_code
    }

    /// Get task TID
    pub fn tid(&self) -> u64 {
        self.tid
    }
}
