//! Types related to task management

use core::iter::Once;

use alloc::vec::Vec;

use crate::config::MAX_SYSCALL_NUM;

use super::TaskContext;

#[derive(Clone)]
/// task control block structure
pub struct TaskControlBlock {
    pub task_status: TaskStatus,
    pub task_cx: TaskContext,
    // LAB1: Add whatever you need about the Task.
    pub syscall_times: Vec<u32>,
    pub start_time: usize,
    pub started: bool,
}

#[derive(Copy, Clone, PartialEq)]
/// task status: UnInit, Ready, Running, Exited
pub enum TaskStatus {
    UnInit,
    Ready,
    Running,
    Exited,
}

pub struct TaskInfo {
    pub status: TaskStatus,
    pub syscall_times: [u32; MAX_SYSCALL_NUM],
    pub time: usize,
}
