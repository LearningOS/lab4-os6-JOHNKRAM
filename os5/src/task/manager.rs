//! Implementation of [`TaskManager`]
//!
//! It is only used to manage processes and schedule process based on ready queue.
//! Other CPU process monitoring functions are in Processor.

use core::cmp::Ordering;

use super::TaskControlBlock;
use crate::sync::UPSafeCell;
use alloc::collections::BinaryHeap;
use alloc::sync::Arc;
use lazy_static::*;

struct HeapElement(Arc<TaskControlBlock>);

impl Ord for HeapElement {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0
            .inner_inclusive_access()
            .pass
            .cmp(&other.0.inner_inclusive_access().pass)
    }
}

impl Eq for HeapElement {}

impl PartialOrd for HeapElement {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for HeapElement {
    fn eq(&self, other: &Self) -> bool {
        self.0
            .inner_inclusive_access()
            .pass
            .eq(&other.0.inner_inclusive_access().pass)
    }
}

pub struct TaskManager {
    ready_queue: BinaryHeap<HeapElement>,
}

// YOUR JOB: FIFO->Stride
/// A simple FIFO scheduler.
impl TaskManager {
    pub fn new() -> Self {
        Self {
            ready_queue: BinaryHeap::<HeapElement>::new(),
        }
    }
    /// Add process back to ready queue
    pub fn add(&mut self, task: Arc<TaskControlBlock>) {
        self.ready_queue.push(HeapElement(task));
    }
    /// Take a process out of the ready queue
    pub fn fetch(&mut self) -> Option<Arc<TaskControlBlock>> {
        self.ready_queue.pop().map(|e| e.0)
    }
}

lazy_static! {
    /// TASK_MANAGER instance through lazy_static!
    pub static ref TASK_MANAGER: UPSafeCell<TaskManager> =
        unsafe { UPSafeCell::new(TaskManager::new()) };
}

pub fn add_task(task: Arc<TaskControlBlock>) {
    TASK_MANAGER.exclusive_access().add(task);
}

pub fn fetch_task() -> Option<Arc<TaskControlBlock>> {
    TASK_MANAGER.exclusive_access().fetch()
}
