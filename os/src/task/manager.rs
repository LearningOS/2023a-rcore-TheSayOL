//!Implementation of [`TaskManager`]
use super::TaskControlBlock;
use crate::sync::UPSafeCell;
use alloc::sync::Arc;
use alloc::vec::Vec;
use lazy_static::*;
///A array of `TaskControlBlock` that is thread-safe
pub struct TaskManager {
    ready_queue: Vec<Arc<TaskControlBlock>>,
}

/// A simple FIFO scheduler.
impl TaskManager {
    ///Creat an empty TaskManager
    pub fn new() -> Self {
        Self {
            ready_queue: Vec::new(),
        }
    }
    /// Add process back to ready queue
    pub fn add(&mut self, task: Arc<TaskControlBlock>) {
        self.ready_queue.push(task);
    }
    /// Take a process out of the ready queue
    pub fn fetch(&mut self) -> Option<Arc<TaskControlBlock>> {
        if self.ready_queue.len() == 0 {
            return None;
        }
        // -------- 从准备队列里, 找到 stride 最小的 tcb -----------

        // 遍历队列, 找到最大步长
        let mut max_pass = 0;
        for tcb in &self.ready_queue { 
            max_pass = max_pass.max(tcb.inner_exclusive_access().pass);
        }
        // 遍历队列, 找到最小 stride
        let mut index = 0;
        let mut ret: Arc<TaskControlBlock> = self.ready_queue[index].clone();
        let mut min_stride = crate::config::BIG_STRIDE;

        
        for i in 0..self.ready_queue.len()  {
            let tcb = &self.ready_queue[i];
            let stride = tcb.inner_exclusive_access().stride;
            
            // - 若 stride == min_stride: 步长一样, 判断优先级高者(数值大), 成为新的 min_stride
            if stride - min_stride == 0 && tcb.inner_exclusive_access().prio > ret.inner_exclusive_access().prio {
                ret = tcb.clone(); 
                index = i;
            }
            // - 若 stride - min_stride > 最大步长: stride 小, 成为新的 min
            else if stride - min_stride > crate::config::BIG_STRIDE-max_pass+1 {
                min_stride = stride;
                ret = tcb.clone();
                index = i;
            } 
        }

        // -------- 从准备队列里, 找到 stride 最小的 tcb -----------

        // 将这个 tcb 出队
        self.ready_queue.remove(index);

        // stride 加上 pass
        let pass = ret.inner_exclusive_access().pass;
        ret.inner_exclusive_access().stride = min_stride.overflowing_add(pass).0;

        Some(ret) 
    }
}

lazy_static! {
    /// TASK_MANAGER instance through lazy_static!
    pub static ref TASK_MANAGER: UPSafeCell<TaskManager> =
        unsafe { UPSafeCell::new(TaskManager::new()) };
}

/// Add process to ready queue
pub fn add_task(task: Arc<TaskControlBlock>) {
    //trace!("kernel: TaskManager::add_task");
    TASK_MANAGER.exclusive_access().add(task);
}

/// Take a process out of the ready queue
pub fn fetch_task() -> Option<Arc<TaskControlBlock>> {
    //trace!("kernel: TaskManager::fetch_task");
    TASK_MANAGER.exclusive_access().fetch()
}
