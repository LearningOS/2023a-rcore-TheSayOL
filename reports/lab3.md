- [ch5](#ch5)
  - [荣誉准则](#荣誉准则)
  - [功能总结](#功能总结)
  - [问题: stride 算法深入](#问题-stride-算法深入)

----


# ch5

## 荣誉准则

1. 在完成本次实验的过程（含此前学习的过程）中，我曾分别与 以下各位 就（与本次实验相关的）以下方面做过交流，还在代码中对应的位置以注释形式记录了具体的交流对象及内容：

    > 无

2. 此外，我也参考了 以下资料 ，还在代码中对应的位置以注释形式记录了具体的参考来源及内容：

    > 无

3. 我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。 我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。

4. 我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。 我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。 我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。 我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按“-100”分计。


## 功能总结

重写`sys_get_time, sys_task_info, sys_mmap, sys_munmap`, 使其可以正常工作

实现`sys_spawn`: 创建一个子进程
- 参数: 
  - `path`: 程序路径名
- 返回 pid 成功, -1 失败

实现`set_prio`: 为当前进程设置优先级
- 参数:
  - `prio`: 优先级
- 返回 `prio` 成功, -1失败

实现`stride`调度算法: 根据进程优先级进行调度


## 问题: stride 算法深入

> stride 算法原理非常简单，但是有一个比较大的问题。例如两个 pass = 10 的进程，使用 8bit 无符号整形储存 stride， p1.stride = 255, p2.stride = 250，在 p2 执行一个时间片后，理论上下一次应该 p1 执行。
> 
> 实际情况是轮到 p1 执行吗？为什么？

实际上可能不是 p1 执行
- p2 的下一个 stride 为: `250 + 10 = 4`(溢出)
- 如果直接判断大小, 比如 `p1.stride > p2.stride`, 会让 p2 执行

> 我们之前要求进程优先级 >= 2 其实就是为了解决这个问题。可以证明， 在不考虑溢出的情况下 , 在进程优先级全部 >= 2 的情况下，如果严格按照算法执行，那么 STRIDE_MAX – STRIDE_MIN <= BigStride / 2。

> 为什么？尝试简单说明（不要求严格证明）。

考虑命题`STRIDE_MAX – STRIDE_MIN <= PASS_MAX`为真, 说明如下:
- 假设某次调度使进程 A 上处理机, 且下一次调度时调度 B
- 此时, `A.stride > B.stride`, 而且两者差值最大的情况应该是 
  - 在调度 A 时, `A.stride == B.stride && A.pass == PASS_MAX`
  - 在调度 A 后, `A.stride == STRIDE_MAX`
- 此时, `STRIDE_MAX – STRIDE_MIN == PASS_MAX`
- 显然, 只会有`STRIDE_MAX – STRIDE_MIN < PASS_MAX`
- 所以, 命题为真

根据`pass = BigStride / prio`来看, `pass`无论如何都不会大于`BigStride / 2`, 所以有
```
STRIDE_MAX – STRIDE_MIN <= PASS_MAX <= BigStride / 2
```

> 已知以上结论，考虑溢出的情况下，可以为 Stride 设计特别的比较器，让 BinaryHeap<Stride> 的 pop 方法能返回真正最小的 Stride。补全下列代码中的 partial_cmp 函数，假设两个 Stride 永远不会相等。
```rust
use core::cmp::Ordering;

struct Stride(u64);

impl PartialOrd for Stride {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // ...
    }
}

impl PartialEq for Stride {
    fn eq(&self, other: &Self) -> bool {
        false
    }
}
```
> TIPS: 使用 8 bits 存储 stride, BigStride = 255, 则: (125 < 255) == false, (129 < 255) == true.

补全如下
```rust
impl PartialOrd for Stride {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.0.overflowing_sub(other.0).0 <= u64::MAX / 2{
            Some(Ordering::Greater)
        } else {
            Some(Ordering::Less)
        }
    }
}
```

