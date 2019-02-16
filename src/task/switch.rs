use alloc::vec::Vec;
use core::ops::DerefMut;
use core::ptr::null_mut;

use super::task::TaskStatus;
use super::{set_current_tid, tasks_mut, Task};

/// Scheduler
pub fn switch(tick: u64) {
    // Wait for 5 ticks
    if tick % 5 != 0 {
        return;
    }

    let mut to_ptr: *mut Task = null_mut();
    // Find next task
    // Use round-robin scheduling
    {
        let mut tasks = tasks_mut();
        {
            // we must have a currently running task
            // otherwise panic
            let current = tasks.current();
            if current.is_none() {
                panic!("No running task, scheduler aborted.");
            }
        }

        // remove terminated tasks
        {
            let mut died_tasks: Vec<u64> = Vec::new();
            for (tid, task_lock) in tasks.iter() {
                let task = task_lock.read();
                if task.died() {
                    died_tasks.push(*tid);
                }
            }
            for tid in died_tasks.iter() {
                tasks.remove(tid);
            }
        }

        // Find next available task
        {
            let check_task = |task: &mut Task| -> bool { task.standby() };

            let current_id = super::TASK_ID.read();
            for (tid, task_lock) in tasks.iter() {
                if *tid > *current_id {
                    let mut task = task_lock.write();
                    if check_task(&mut task) {
                        to_ptr = task.deref_mut() as *mut Task;
                    }
                }
            }

            if to_ptr == null_mut() {
                for (tid, task_lock) in tasks.iter() {
                    if *tid < *current_id {
                        let mut task = task_lock.write();
                        if check_task(&mut task) {
                            to_ptr = task.deref_mut() as *mut Task;
                        }
                    }
                }
            }

            // Set running task status
            if to_ptr != null_mut() {
                let current_lock = tasks.current().unwrap();
                let mut current = current_lock.write();
                current.status = TaskStatus::Ready;
            }
        }
    }

    if to_ptr == null_mut() {
        println!("No scheduled task, returning to original.");
    } else {
        unsafe {
            println!("Switching to task {}", (*to_ptr).tid());
            (*to_ptr).status = TaskStatus::Running;
            set_current_tid((*to_ptr).tid());
            (*to_ptr).context.switch_to();
        }
    }
}
