use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use spin::RwLock;

use super::task::Task;

const MAX_TASKS: u64 = 10000;

pub struct TaskList {
    map: BTreeMap<u64, Arc<RwLock<Task>>>,
    next_id: u64,
}

impl TaskList {
    pub fn new() -> Self {
        Self {
            map: BTreeMap::new(),
            next_id: 1,
        }
    }

    pub fn current(&self) -> Option<&Arc<RwLock<Task>>> {
        self.map.get(&super::TASK_ID.read())
    }

    pub fn new_task(&mut self) -> Result<&Arc<RwLock<Task>>, ::common::error::Error> {
        // Find next TID
        let mut alloc_id = self.next_id;
        loop {
            if !self.map.contains_key(&alloc_id) {
                break;
            }
            alloc_id += 1;
            if alloc_id >= MAX_TASKS {
                alloc_id = 1;
            }
        }
        self.next_id = alloc_id;

        assert!(
            self.map
                .insert(alloc_id, Arc::new(RwLock::new(Task::new(alloc_id))))
                .is_none()
        );
        Ok(self.map.get(&alloc_id).unwrap())
    }

    pub fn iter(&self) -> ::alloc::collections::btree_map::Iter<u64, Arc<RwLock<Task>>> {
        self.map.iter()
    }

    pub fn remove(&mut self, tid: &u64) -> Option<Arc<RwLock<Task>>> {
        self.map.remove(tid)
    }
}
