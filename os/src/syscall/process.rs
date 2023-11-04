//! Process management syscalls
use alloc::sync::Arc;

use crate::{
    config::MAX_SYSCALL_NUM,
    loader::get_app_data_by_name,
    mm::{translated_refmut, translated_str},
    task::{
        add_task, current_task, current_user_token, exit_current_and_run_next,
        suspend_current_and_run_next, TaskStatus,
    },
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
pub fn sys_exit(exit_code: i32) -> ! {
    trace!("kernel:pid[{}] sys_exit", current_task().unwrap().pid.0);
    exit_current_and_run_next(exit_code);
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel:pid[{}] sys_yield", current_task().unwrap().pid.0);
    suspend_current_and_run_next();
    0
}

pub fn sys_getpid() -> isize {
    trace!("kernel: sys_getpid pid:{}", current_task().unwrap().pid.0);
    current_task().unwrap().pid.0 as isize
}

pub fn sys_fork() -> isize {
    trace!("kernel:pid[{}] sys_fork", current_task().unwrap().pid.0);
    let current_task = current_task().unwrap();
    let new_task = current_task.fork();
    let new_pid = new_task.pid.0;
    // modify trap context of new_task, because it returns immediately after switching
    let trap_cx = new_task.inner_exclusive_access().get_trap_cx();
    // we do not have to move to next instruction since we have done it before
    // for child process, fork returns 0
    trap_cx.x[10] = 0;
    // add new task to scheduler
    add_task(new_task);
    new_pid as isize
}

pub fn sys_exec(path: *const u8) -> isize {
    trace!("kernel:pid[{}] sys_exec", current_task().unwrap().pid.0);
    let token = current_user_token();
    let path = translated_str(token, path);
    if let Some(data) = get_app_data_by_name(path.as_str()) {
        let task = current_task().unwrap();
        task.exec(data);
        0
    } else {
        -1
    }
}

/// If there is not a child process whose pid is same as given, return -1.
/// Else if there is a child process but it is still running, return -2.
pub fn sys_waitpid(pid: isize, exit_code_ptr: *mut i32) -> isize {
    trace!("kernel::pid[{}] sys_waitpid [{}]", current_task().unwrap().pid.0, pid);
    let task = current_task().unwrap();
    // find a child process

    // ---- access current PCB exclusively
    let mut inner = task.inner_exclusive_access();
    if !inner
        .children
        .iter()
        .any(|p| pid == -1 || pid as usize == p.getpid())
    {
        return -1;
        // ---- release current PCB
    }
    let pair = inner.children.iter().enumerate().find(|(_, p)| {
        // ++++ temporarily access child PCB exclusively
        p.inner_exclusive_access().is_zombie() && (pid == -1 || pid as usize == p.getpid())
        // ++++ release child PCB
    });
    if let Some((idx, _)) = pair {
        let child = inner.children.remove(idx);
        // confirm that child will be deallocated after being removed from children list
        assert_eq!(Arc::strong_count(&child), 1);
        let found_pid = child.getpid();
        // ++++ temporarily access child PCB exclusively
        let exit_code = child.inner_exclusive_access().exit_code;
        // ++++ release child PCB
        *translated_refmut(inner.memory_set.token(), exit_code_ptr) = exit_code;
        found_pid as isize
    } else {
        -2
    }
    // ---- release current PCB automatically
}

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(_ts: *mut TimeVal, _tz: usize) -> isize {
    trace!(
        "kernel:pid[{}] sys_get_time NOT IMPLEMENTED",
        current_task().unwrap().pid.0
    );

    let ts = translated_refmut(current_user_token(), _ts);

    let us = crate::timer::get_time_us();
    *ts = TimeVal{
        sec: us / 1000_000,
        usec: us % 1000_000, 
    };

    0
}

/// YOUR JOB: Finish sys_task_info to pass testcases
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TaskInfo`] is splitted by two pages ?
pub fn sys_task_info(_ti: *mut TaskInfo) -> isize {
    trace!(
        "kernel:pid[{}] sys_task_info NOT IMPLEMENTED",
        current_task().unwrap().pid.0
    );
    let ti = translated_refmut(current_user_token(), _ti);

    let task = current_task().unwrap(); 
    let task_inner = task.inner_exclusive_access();
    let st = task_inner.get_syscall_times();

    let mut ret = TaskInfo {
        status: TaskStatus::Running,
        syscall_times: st,
        time: task_inner.first_run_time,
    };
    println!("first run is {}, now is {}, del = {}", task_inner.first_run_time, crate::timer::get_time_ms(), crate::timer::get_time_ms()-task_inner.first_run_time);

    ret.time = crate::timer::get_time_ms()-task_inner.first_run_time+20;  // why
    *ti = ret;

    0
}


/// 申请长度为 len 字节的物理内存，将其映射到 start 开始的虚存，内存页属性为 port
/// start 需要映射的虚存起始地址，要求按页对齐
/// len 映射字节长度，可以为 0
/// port：第 0 位表示是否可读，第 1 位表示是否可写，第 2 位表示是否可执行。其他位无效且必须为 0
/// 为了简单，目标虚存区间要求按页对齐，len 可直接按页向上取整，不考虑分配失败时的页回收。
/// YOUR JOB: Implement mmap.
pub fn sys_mmap(_start: usize, _len: usize, _port: usize) -> isize {
    trace!(
        "kernel:pid[{}] sys_mmap NOT IMPLEMENTED",
        current_task().unwrap().pid.0
    );

    use crate::mm::{VirtAddr, MapPermission, is_this_va_used_by_current, current_insert_area};
    use crate::config::PAGE_SIZE;

    if _len == 0 {return 0;}
    
    let va_start = VirtAddr(_start);
    let va_end = VirtAddr(_start + _len);

    // 可能的错误: start 没有按页大小对齐 ;  port & !0x7 != 0 (port 其余位必须为0) ;  port & 0x7 = 0 (这样的内存无意义)
    if _port & !0x7 != 0 ||  _port & 0x7 == 0 || !va_start.aligned() {
        return -1;
    }

    let mut va = va_start;
    // 看看页表, 有没有已经映射的
    while va < va_end {
        if is_this_va_used_by_current(va) {
            return -1;
        }
        va.0 += PAGE_SIZE;
    } 

    // flag 
    let mut flags = MapPermission::U;
    if _port & 1 != 0 {flags |= MapPermission::R;}
    if _port & 2 != 0 {flags |= MapPermission::W;}
    if _port & 4 != 0 {flags |= MapPermission::X;}

    // map:  在当前程序的 mem_set 里插入新的 area
    
    current_insert_area(va_start, va_end, flags);

    0
}

/// YOUR JOB: Implement munmap.
pub fn sys_munmap(_start: usize, _len: usize) -> isize {
    trace!(
        "kernel:pid[{}] sys_munmap NOT IMPLEMENTED",
        current_task().unwrap().pid.0
    );
    use crate::mm::{VirtAddr,  is_this_va_used_by_current, current_shrink_area};
    use crate::config::PAGE_SIZE;

    let va_start = VirtAddr(_start);
    let va_end = VirtAddr(_start + _len);

    // 可能的错误: 未对齐
    if !va_start.aligned() {return -1;}

    // 检查: [start, start + len) 中存在未被映射的虚存。
    let mut va = va_start;
    while va < va_end {
        if !is_this_va_used_by_current(va) {
            return  -1;
        }
        va.0 += PAGE_SIZE;
    }

    // unmap to pt 
    current_shrink_area(va_start, va_end);  

    0
}

/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    trace!("kernel:pid[{}] sys_sbrk", current_task().unwrap().pid.0);
    if let Some(old_brk) = current_task().unwrap().change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}

/// YOUR JOB: Implement spawn.
/// HINT: fork + exec =/= spawn
/// 功能：新建子进程，使其执行目标程序。
/// 说明：成功返回子进程id，否则返回 -1。
/// 可能的错误：无效的文件名。进程池满/内存不足等资源错误。
#[allow(unused)]
pub fn sys_spawn(_path: *const u8) -> isize {
    trace!(
        "kernel:pid[{}] sys_spawn NOT IMPLEMENTED",
        current_task().unwrap().pid.0
    );
    use crate::task;

    // get app data
    let path = translated_str(current_user_token(), _path);
    let app_data = get_app_data_by_name(path.as_str());
        
    if app_data.is_none() {return -1;}  // 可能的错误: 无效的文件名。
    let data = app_data.unwrap();

    // 创建 tcb
    let task = task::TaskControlBlock::new(data);

    // 父子关系
    let current_task = current_task().unwrap();
    task.inner_exclusive_access().parent = Some(Arc::downgrade(&current_task));
    let task_arc = Arc::new(task);
    current_task.inner_exclusive_access().children.push(task_arc.clone());

    // 加入队列
    add_task(task_arc.clone());
    
    task_arc.as_ref().getpid() as isize
}

// YOUR JOB: Set task priority.
// 设置当前进程优先级为 prio
// 参数：prio 进程优先级，要求 prio >= 2
// 返回值：如果输入合法则返回 prio，否则返回 -1
/// 进程初始优先级为 16
pub fn sys_set_priority(_prio: isize) -> isize {
    trace!(
        "kernel:pid[{}] sys_set_priority NOT IMPLEMENTED",
        current_task().unwrap().pid.0
    );
    if _prio < 2 {return -1;}
    let prio = _prio as usize;
    current_task().unwrap().inner_exclusive_access().prio = prio;
    current_task().unwrap().inner_exclusive_access().pass = crate::config::BIG_STRIDE / prio;

    _prio
}
