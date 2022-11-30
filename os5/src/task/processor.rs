//! Implementation of [`Processor`] and Intersection of control flow
//!
//! Here, the continuous operation of user apps in CPU is maintained,
//! the current running state of CPU is recorded,
//! and the replacement and transfer of control flow of different applications are executed.

use core::convert::TryInto;

use super::{fetch_task, TaskStatus};
use super::{TaskContext, TaskControlBlock};
use super::{TaskInfo, __switch};
use crate::mm::VirtAddr;
use crate::sync::UPSafeCell;
use crate::timer::get_time_us;
use crate::trap::TrapContext;
use alloc::sync::Arc;
use lazy_static::*;

/// Processor management structure
pub struct Processor {
    /// The task currently executing on the current processor
    current: Option<Arc<TaskControlBlock>>,
    /// The basic control flow of each core, helping to select and switch process
    idle_task_cx: TaskContext,
}

impl Processor {
    pub fn new() -> Self {
        Self {
            current: None,
            idle_task_cx: TaskContext::zero_init(),
        }
    }
    fn get_idle_task_cx_ptr(&mut self) -> *mut TaskContext {
        &mut self.idle_task_cx as *mut _
    }
    pub fn take_current(&mut self) -> Option<Arc<TaskControlBlock>> {
        self.current.take()
    }
    pub fn current(&self) -> Option<Arc<TaskControlBlock>> {
        self.current.as_ref().map(|task| Arc::clone(task))
    }
}

lazy_static! {
    /// PROCESSOR instance through lazy_static!
    pub static ref PROCESSOR: UPSafeCell<Processor> = unsafe { UPSafeCell::new(Processor::new()) };
}

/// The main part of process execution and scheduling
///
/// Loop fetch_task to get the process that needs to run,
/// and switch the process through __switch
pub fn run_tasks() {
    loop {
        let mut processor = PROCESSOR.exclusive_access();
        if let Some(task) = fetch_task() {
            let idle_task_cx_ptr = processor.get_idle_task_cx_ptr();
            // access coming task TCB exclusively
            let mut task_inner = task.inner_exclusive_access();
            let next_task_cx_ptr = &task_inner.task_cx as *const TaskContext;
            task_inner.task_status = TaskStatus::Running;
            if !task_inner.started {
                task_inner.start_time = get_time_us();
                task_inner.started = true;
            }
            let prio = task_inner.prio;
            task_inner.pass.stride(prio);
            drop(task_inner);
            // release coming task TCB manually
            processor.current = Some(task);
            // release processor manually
            drop(processor);
            unsafe {
                __switch(idle_task_cx_ptr, next_task_cx_ptr);
            }
        }
    }
}

/// Get current task through take, leaving a None in its place
pub fn take_current_task() -> Option<Arc<TaskControlBlock>> {
    PROCESSOR.exclusive_access().take_current()
}

/// Get a copy of the current task
pub fn current_task() -> Option<Arc<TaskControlBlock>> {
    PROCESSOR.exclusive_access().current()
}

/// Get token of the address space of current task
pub fn current_user_token() -> usize {
    let task = current_task().unwrap();
    let token = task.inner_exclusive_access().get_user_token();
    token
}

/// Get the mutable reference to trap context of current task
pub fn current_trap_cx() -> &'static mut TrapContext {
    current_task()
        .unwrap()
        .inner_exclusive_access()
        .get_trap_cx()
}

pub fn inc_task_syscall_times(syscall_id: usize) {
    current_task()
        .unwrap()
        .inner_exclusive_access()
        .syscall_times[syscall_id] += 1;
}

pub fn get_current_task_info() -> TaskInfo {
    let task = current_task().unwrap();
    let status = task.inner_inclusive_access().task_status;
    let syscall_times = task
        .inner_inclusive_access()
        .syscall_times
        .as_slice()
        .try_into()
        .unwrap();
    let time = (get_time_us() - task.inner_inclusive_access().start_time) / 1000;
    TaskInfo {
        status,
        syscall_times,
        time,
    }
}

pub fn mmap(start_va: VirtAddr, end_va: VirtAddr, port: u8) -> isize {
    current_task()
        .unwrap()
        .inner_exclusive_access()
        .memory_set
        .map(start_va, end_va, port)
}

pub fn munmap(start_va: VirtAddr, end_va: VirtAddr) -> isize {
    current_task()
        .unwrap()
        .inner_exclusive_access()
        .memory_set
        .unmap(start_va, end_va)
}

pub fn set_current_task_prio(prio: u64) {
    current_task().unwrap().inner_exclusive_access().prio = prio;
}

/// Return to idle control flow for new scheduling
pub fn schedule(switched_task_cx_ptr: *mut TaskContext) {
    let mut processor = PROCESSOR.exclusive_access();
    let idle_task_cx_ptr = processor.get_idle_task_cx_ptr();
    drop(processor);
    unsafe {
        __switch(switched_task_cx_ptr, idle_task_cx_ptr);
    }
}
