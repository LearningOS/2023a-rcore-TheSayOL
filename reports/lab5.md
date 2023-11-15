

----


# ch8

## 荣誉准则

1. 在完成本次实验的过程（含此前学习的过程）中，我曾分别与 以下各位 就（与本次实验相关的）以下方面做过交流，还在代码中对应的位置以注释形式记录了具体的交流对象及内容：

    > 无

2. 此外，我也参考了 以下资料 ，还在代码中对应的位置以注释形式记录了具体的参考来源及内容：

    > 无

3. 我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。 我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。

4. 我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。 我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。 我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。 我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按“-100”分计。


## 功能总结

实现死锁检测功能
- 如果发现某次请求资源会导致死锁, 将拒绝, 返回 -0xdead


## 问答作业

### 1

> 在我们的多线程实现中，当主线程 (即 0 号线程) 退出时，视为整个进程退出， 此时需要结束该进程管理的所有线程并回收其资源。 
> - 需要回收的资源有哪些？ 
> - 其他线程的 TaskControlBlock 可能在哪些位置被引用，分别是否需要回收，为什么？

主线程退出时, 需要回收整个进程的资源
- 进程使用的 pcb, tcb 被内核用于管理的资源
- 进程申请的内存, 比如用于打开文件进行读写的内存

其他线程的 tcb 会被引用于各种任务队列, 比如
- 准备队列: 需要将 tcb 从队列中删除
- 锁的等待队列: 不需要回收, 因为锁也在 pcb 里, 会随着 pcb 的销毁而销毁

### 2

> 对比以下两种 Mutex.unlock 的实现，二者有什么区别？这些区别可能会导致什么问题？

```rust
impl Mutex for Mutex1 {
    fn unlock(&self) {
        let mut mutex_inner = self.inner.exclusive_access();
        assert!(mutex_inner.locked);
        mutex_inner.locked = false;
        if let Some(waking_task) = mutex_inner.wait_queue.pop_front() {   
            add_task(waking_task);
        }
    }
}

impl Mutex for Mutex2 {
    fn unlock(&self) {
        let mut mutex_inner = self.inner.exclusive_access();
        assert!(mutex_inner.locked);
        if let Some(waking_task) = mutex_inner.wait_queue.pop_front() {
            add_task(waking_task);
        } else {
            mutex_inner.locked = false;
        }
    }
}
```

如果等待队列里没有线程, 这两个方法是等价的

如果等待队列有线程, 两个方法都会从队列里取出一个线程 A, 放入准备队列

区别在于: 方法 1 还会将锁解开, 这会导致插队的发生
- 刚刚被从等待队列里取出的线程 A, 被放到了准备队列的末尾
- 此时如果有新线程申请锁, 会直接成功, 因为锁是解开的状态