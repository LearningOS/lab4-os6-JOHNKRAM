//! Process management syscalls

use crate::mm::{translate_pointer, VirtAddr};
//use crate::config::MAX_SYSCALL_NUM;
pub use crate::task::TaskInfo;
use crate::task::{
    current_user_token, exit_current_and_run_next, get_current_task_info, mmap, munmap,
    suspend_current_and_run_next,
};
use crate::timer::get_time_us;

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

pub fn sys_exit(exit_code: i32) -> ! {
    info!("[kernel] Application exited with code {}", exit_code);
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    suspend_current_and_run_next();
    0
}

// YOUR JOB: 引入虚地址后重写 sys_get_time
pub fn sys_get_time(_ts: *mut TimeVal, _tz: usize) -> isize {
    let us = get_time_us();
    // unsafe {
    //     *ts = TimeVal {
    //         sec: us / 1_000_000,
    //         usec: us % 1_000_000,
    //     };
    // }
    let ts = translate_pointer(current_user_token(), _ts);
    unsafe {
        *ts = TimeVal {
            sec: us / 1_000_000,
            usec: us % 1_000_000,
        };
    }
    0
}

// CLUE: 从 ch4 开始不再对调度算法进行测试~
pub fn sys_set_priority(_prio: isize) -> isize {
    -1
}

// YOUR JOB: 扩展内核以实现 sys_mmap 和 sys_munmap
pub fn sys_mmap(_start: usize, _len: usize, _port: usize) -> isize {
    let start_va = VirtAddr::from(_start);
    if start_va.page_offset() != 0 {
        return -1;
    }
    let port = _port & 7;
    if port == 0 || port != _port {
        return -1;
    }
    let end_va = VirtAddr::from(_start + _len);
    mmap(start_va, end_va, port as u8)
}

pub fn sys_munmap(_start: usize, _len: usize) -> isize {
    let start_va = VirtAddr::from(_start);
    if start_va.page_offset() != 0 {
        return -1;
    }
    let end_va = VirtAddr::from(_start + _len);
    munmap(start_va, end_va)
}

// YOUR JOB: 引入虚地址后重写 sys_task_info
pub fn sys_task_info(ti: *mut TaskInfo) -> isize {
    let ti = translate_pointer(current_user_token(), ti);
    unsafe { *ti = get_current_task_info() }
    0
}
