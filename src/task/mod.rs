mod list;
mod switch;
mod task;

use arch;
use spin::{Once, RwLock, RwLockReadGuard, RwLockWriteGuard};

pub use self::list::TaskList;
pub use self::task::{Task, TaskStatus};

static TASK_LIST: Once<RwLock<TaskList>> = Once::new();
static TASK_ID: RwLock<u64> = RwLock::new(0);

pub fn tasks() -> RwLockReadGuard<'static, TaskList> {
    TASK_LIST.call_once(|| RwLock::new(TaskList::new())).read()
}

pub fn tasks_mut() -> RwLockWriteGuard<'static, TaskList> {
    TASK_LIST.call_once(|| RwLock::new(TaskList::new())).write()
}

pub fn current_tid() -> u64 {
    *TASK_ID.read()
}

pub fn set_current_tid(tid: u64) {
    *TASK_ID.write() = tid;
}

pub fn init() {
    arch::register_scheduler(self::switch::switch).expect("Failed to register scheduler");
}
