// use crate::task::{current_task, current_process};

use alloc::vec::Vec;

// use crate::task::{ current_process};

/// 保存所有互斥资源的信息
///
/// 随进程创建时存入 pcb, mutex 和 sem 各一个
#[allow(unused)]
pub struct DeadLockDetector {
    /// 是否启用检测
    enable: bool,
    /// 各资源目前可用数目
    available: Vec<usize>,
    /// 各资源已分配数目
    allocation: Vec<Vec<usize>>,
    /// 各资源请求数目
    need: Vec<Vec<usize>>,
}

#[allow(unused)]
impl DeadLockDetector {
    /// 新建一个, 放在 pcb 里, 所以建立 pcb 时需要一起建立一个
    pub fn new() -> Self {
        // info!("new: pid = {}", current_process().getpid());
        // 主线程没有创建过程, 所以需要额外创建
        let mut allocation = Vec::new();
        allocation.push(Vec::new());
        let mut need = Vec::new();
        need.push(Vec::new());

        // 生成 self
        let ret = Self {
            enable: false,
            available: Vec::new(),
            allocation,
            need,
        };
        // info!("new: allocation = {:?}", ret.allocation);
        // info!("new: need = {:?}", ret.need);
        ret
    }

    /// 创建线程时使用, 修改 need 和 allocation
    pub fn set_tid(&mut self, tid: usize) {
        info!("set_tid: tid = {}", tid);
        while self.need.get(tid).is_none() {
            let mut v = Vec::new();
            let mut v2 = Vec::new();
            while v.len() < self.available.len() {
                v.push(0);
                v2.push(0);
            }
            self.need.push(v);
            self.allocation.push(v2);
        }
        info!("set_tid: allocation = {:?}", self.allocation);
        info!("set_tid: need = {:?}", self.need);}

    /// 当创建一个锁时调用, 新增一项资源
    ///
    /// 简便起见可对 mutex 和 semaphore 分别进行检测，无需考虑二者 (以及 waittid 等) 混合使用导致的死锁。
    pub fn update_available(&mut self, res_id: usize, num: usize) {
        // info!("update_avaiable: res_id = {}, num = {}", res_id, num);
        // 因为不能删除锁, 所以 res_id 必然是最新的
        // assert_eq!(self.need.len(), res_id);
        // assert_eq!(self.available[0].len(), res_id);
        // assert_eq!(self.allocation[0].len(), res_id);

        // 设置 available, 新增一项
        self.available.push(num);

        // 修改 need, allocation 的资源种类数
        for i in 0..self.need.len() {
            self.need[i].push(0);
            self.allocation[i].push(0);
        }
    }

    /// 上锁时调用
    pub fn aquire_one(&mut self, res_id: usize, tid: usize) {
        // info!("aquire_one: res_id = {}, tid = {}", res_id, tid);
        // info!("aquire_one: available = {:?}", self.available);
        // info!("aquire_one: allocation = {:?}", self.allocation);
        // info!("aquire_one: need = {:?}", self.need);
        // 可用, available -1 并且 allocation +1
        if self.available[res_id] != 0 {
            self.available[res_id] -= 1;
            self.allocation[tid][res_id] += 1;
        // 无可用, need + 1 
        } else {
            self.need[tid][res_id] += 1;
        }
    }

    /// 解锁时调用
    pub fn release_one(&mut self, res_id: usize, tid: usize) {
        // allocation -1, avail +1
        self.available[res_id] += 1;
        self.allocation[tid][res_id] -= 1;
    }

    /// 如果没启用检测, 始终返回 true
    pub fn detect_deadlock(&mut self, tid: usize, res_id: usize) -> bool {
        info!("detect_deadlock: enable = {}", self.enable);
        if self.enable {
            self._detect_deadlock(tid, res_id)
        } else {
            true
        }
    }

    /// 启动
    pub fn set_enable(&mut self, enable: bool) {
        info!("set_enable: para = {}", enable);
        info!("set_enable: enable = {}", self.enable);
        self.enable = enable;
        info!("set_enable: enable = {}", self.enable);
    }

    /// 检测死锁算法
    /// 参数: 线程 id, 请求的资源的 id
    /// 返回: 是否允许本次请求
    fn _detect_deadlock(&mut self, tid: usize, res_id: usize) -> bool {

        info!("------------------_detect_deadlock: --------------------------");
        info!("now is:");
        info!("avail = {:?}", self.available);
        info!("alloc = {:?}", self.allocation);
        info!("need = {:?}", self.need);

        // temporarily need + 1 
        self.need[tid][res_id] += 1;
        info!("need = {:?}", self.need);
        info!("------------------_detect_deadlock: --------------------------");

        // create work and finish 
        let mut work = Vec::new();
        let mut finish = Vec::new();
        for i in self.available.iter() {
            work.push(*i);
        }
        for _ in 0..self.need.len() {
            finish.push(false);
        }

        // loop : try to finish at least one 
        loop {
            info!("loop: work = {:?},", work);
            info!("loop: fini = {:?},", finish);
            // 至少找到一个 = false
            let mut atleast_found_one = false;
            // 遍历 need
            for (index, one_need) in self.need.iter().enumerate() {
                // 已经 finish 的, 跳过
                if finish[index] {
                    continue;
                }
                let mut found = true;
                // 遍历 need[index], 判断是否 <= work
                for (i, n) in one_need.iter().enumerate() {
                    if n > &work[i] {
                        found = false;
                        break;
                    }
                }
                // 如果 need[index] <= work
                if found {
                    atleast_found_one = true;
                    finish[index] = true;
                    let one_allocation = &self.allocation[index];
                    for (i, n) in one_allocation.iter().enumerate() {
                        work[i] += *n;
                    }
                    break;
                }
            }
            info!("one loop done, atleast_found_one = {}", atleast_found_one);
            // 如果一个都没找到 break
            if !atleast_found_one {
                break;
            }
        }
        self.need[tid][res_id] -= 1;

        let ret = finish.iter().all(|x| x == &true);
        info!("ok, ret = {}", ret);

        ret 
    }
}
