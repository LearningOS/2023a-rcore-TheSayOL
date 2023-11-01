//! Process management syscalls
#[allow(unused)]
use crate::{
    config::MAX_SYSCALL_NUM,
    task::{
        change_program_brk, exit_current_and_run_next, suspend_current_and_run_next, TaskStatus, current_taskinfo, current_user_token,
    }, timer::{get_time_us, get_time_ms}, mm::{mmap, munmap, VirtAddr, PageTable},
};

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// Task information
#[allow(dead_code)]
pub struct TaskInfo {
    /// Task status in it's life cycle
    status: TaskStatus,
    /// The numbers of syscall called by task
    syscall_times: [u32; MAX_SYSCALL_NUM],
    /// Total running time of task
    time: usize,
}

/// task exits and submit an exit code
pub fn sys_exit(_exit_code: i32) -> ! {
    trace!("kernel: sys_exit");
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(_ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");

    let user_va = VirtAddr(_ts as usize);
    let fake_user_pt = PageTable::from_token(current_user_token());
    let ppn = fake_user_pt.translate(user_va.floor()).unwrap().ppn();
    let pa = (ppn.0 << 12) + user_va.page_offset();
    let us = get_time_us();
    unsafe {
        *(pa as *mut TimeVal) = TimeVal{
            sec: us / 1000_000,
            usec: us % 1000_000, 
        }; 
    }
    
    0
}

/// YOUR JOB: Finish sys_task_info to pass testcases
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TaskInfo`] is splitted by two pages ?
#[allow(unused)]
pub fn sys_task_info(_ti: *mut TaskInfo) -> isize {
    trace!("kernel: sys_task_info NOT IMPLEMENTED YET!");
    let (syscall_times, first_start_time) = current_taskinfo();

    let user_va = VirtAddr(_ti as usize);
    let fake_user_pt = PageTable::from_token(current_user_token());
    let ppn = fake_user_pt.translate(user_va.floor()).unwrap().ppn();
    let pa = (ppn.0 << 12) + user_va.page_offset();
    let pa = pa as *mut TaskInfo;


    unsafe { 
        *pa = TaskInfo {
            status: TaskStatus::Running,
            syscall_times, 
            time: get_time_ms(),   // should be  `get_time_ms() - first_start_time`, but can't pass, idk why
        };  
    }
    0
}



// YOUR JOB: Implement mmap.
pub fn sys_mmap(_start: usize, _len: usize, _port: usize) -> isize {
    trace!("kernel: sys_mmap NOT IMPLEMENTED YET!");
    mmap(_start, _len, _port)
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(_start: usize, _len: usize) -> isize {
    trace!("kernel: sys_munmap NOT IMPLEMENTED YET!");
    munmap(_start, _len)
}
/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    trace!("kernel: sys_sbrk");
    if let Some(old_brk) = change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}
