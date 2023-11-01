- [lab1: Ch3](#lab1-ch3)
  - [荣誉准则](#荣誉准则)
  - [功能总结](#功能总结)
  - [问题1:](#问题1)
  - [问题2](#问题2)
    - [2.1](#21)
    - [2.2](#22)
    - [2.3](#23)
    - [2.4](#24)
    - [2.5](#25)
    - [2.6](#26)
    - [2.7](#27)

----


# lab1: Ch3

## 荣誉准则

1. 在完成本次实验的过程（含此前学习的过程）中，我曾分别与 以下各位 就（与本次实验相关的）以下方面做过交流，还在代码中对应的位置以注释形式记录了具体的交流对象及内容：

    > 无

2. 此外，我也参考了 以下资料 ，还在代码中对应的位置以注释形式记录了具体的参考来源及内容：

    > 无

3. 我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。 我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。

4. 我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。 我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。 我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。 我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按“-100”分计。


## 功能总结

实现了系统调用`sys_task_info`, 获取当前任务的信息
- 调用号: 410
- 参数: `*mut TaskInfo`
- 返回: 成功`0`, 失败`-1`

具体信息如下: 
- status: 进程状态
  - 在 `task/mod.rs` 里暴露接口, 返回当前进程的状态
- syscall_times: 各系统调用次数
  - 在 tcb 里新增一个数组, 初始化为 0 ; 每次 syscall 的时候, 根据调用号, 数组对应元素 + 1 
  - 在 `task/mod.rs` 里暴露接口, 执行数组对应元素 + 1 
  - 在 `task/mod.rs` 里暴露接口, 返回这个数组
- time: 总运行时间
  - 在 tcb 记录初次运行的时间
  - 在 `task/mod.rs` 里暴露接口, 返回这个数字


## 问题1: 

> 正确进入 U 态后，程序的特征还应有：使用 S 态特权指令，访问 S 态寄存器后会报错。 请同学们可以自行测试这些内容 (运行 Rust 三个 bad 测例 (ch2b_bad_*.rs) ， 注意在编译时至少需要指定 LOG=ERROR 才能观察到内核的报错信息) ， 描述程序出错行为，同时注意注明你使用的 sbi 及其版本。

使用的 SBI 如下
```
RustSBI version 0.3.0-alpha.2, adapting to RISC-V SBI v1.0.0
```

三个 bad 测例行为如下
- `ch2b_bad_address.rs`里, 程序试图向地址为`0x0`的内存写入`0`
- `ch2b_bad_instructions.rs`里, 程序试图使用特权指令`sret`
- `ch2b_bad_register.rs`里, 程序试图读取寄存器`sstatus`

运行`make run LOG=ERROR`, 以上程序均被 OS 杀死, 信息如下
```
[kernel] PageFault in application, bad addr = 0x0, bad instruction = 0x804003c4, kernel killed it.
[kernel] IllegalInstruction in application, kernel killed it.
[kernel] IllegalInstruction in application, kernel killed it.
```


## 问题2 

> 深入理解 trap.S 中两个函数 __alltraps 和 __restore 的作用，并回答如下问题:

### 2.1 

> L40：刚进入 __restore 时，a0 代表了什么值。请指出 __restore 的两种使用情景。

`__restore` 的作用是: 恢复指定程序在用户态运行时上下文

`a0` 作为 `__restore` 的第一个参数, 代表内核栈 `sp`, 指向一个上下文

`__restore` 有两种使用情况
- 操作系统运行第一个进程
- 从 `Trap` 返回

操作系统要让第一个进程上处理机, 需要进行以下: 
- 构造一个特殊的上下文 `cx`, 然后调用 `__restore`, 参数为 `cx` 的地址.
- `__restore` 会将 `sp` 设置为 `cx` 的地址, 将 `cx` 内容逐个写入对应寄存器, 虽然 `cx` 是操作系统构造的, 但也有几个寄存器值得注意: 
  - `sstatus`: 记录`Trap`发生时处在哪个特权级 -- 自然是 `U`
  - `sepc`: 记录触发`Trap`的指令地址, 操作系统构造为, 进程的第一行代码
  - `sscratch`: 保存着用户栈 `sp`, `__restore` 用它恢复 `sp` 为用户 `sp`, 同时也会将当前 `sp` 也就是内核栈 `sp` 写入其中
- `__restore` 恢复所有寄存器后, 执行 `sret`, 成功进入目标进程开始执行

要从 `Trap` 返回, 需要进行以下: 
- 发生 `Trap`, 进入内核态, 调用 `__alltraps`, 
- `__alltraps` 设置 `sp` 为从 `sscratch` 里得到的内核栈 `sp`, 同时将旧的 `sp`(即用户栈 `sp`) 存入 `sscratch`, 之后开始将当前上下文写入内核栈中, 接着调用 `trap_handler`
- `trap_handler` 执行完毕后, 调用 `__restore`
- `__restore` 进行一样的操作, 恢复上下文, 保存内核栈 `sp` 到 `sscratch` 后, `sret`


### 2.2 

> L43-L48：这几行汇编代码特殊处理了哪些寄存器？这些寄存器的的值对于进入用户态有何意义？请分别解释。

```asm
ld t0, 32*8(sp)
ld t1, 33*8(sp)
ld t2, 2*8(sp)
csrw sstatus, t0
csrw sepc, t1
csrw sscratch, t2
```

这几行代码从内存里指定内容写入了`sstatus, sepc, sscratch`寄存器里, 其内容的意义如下:
- `sstatus`: 记录`Trap`发生时处在哪个特权级 
- `sepc`: 记录触发`Trap`的指令地址, 执行`sret`就会跳转到这个地址运行
- `sscratch`: 记录用户/内核栈 `sp`, `__restore` 用它作为中转, 恢复一个`sp`的同时将另一个`sp`写入, 留待以后使用


### 2.3 

> L50-L56：为何跳过了 x2 和 x4？

```asm
ld x1, 1*8(sp)
ld x3, 3*8(sp)
.set n, 5
.rept 27
LOAD_GP %n
.set n, n+1
.endr
```

x2 就是 sp, 不需要专门写入

x4 是 tp(线程指针), 指向 tcb, 而本实验使用一个全局变量维护每个线程的 tcb, 所以不需要保存.


### 2.4

> L60：该指令之后，sp 和 sscratch 中的值分别有什么意义？

```asm
csrrw sp, sscratch, sp
```

该指令实现了 sp 和 sscratch 里的值的交换

在第 60 行, 指令执行之后, sp 指向用户栈, sscratch 指向内核栈 


### 2.5 

> __restore：中发生状态切换在哪一条指令？为何该指令执行之后会进入用户态？

状态切换发生在`sret`, 其会根据`sstatus`, 切换到对应的特权级
- `sstatus`: 记录`Trap`发生时处在哪个特权级 


### 2.6

> L13：该指令之后，sp 和 sscratch 中的值分别有什么意义？

```asm
csrrw sp, sscratch, sp
```

该指令实现了 sp 和 sscratch 里的值的交换

在第 13 行, 指令执行之后, sp 指向内核栈, sscratch 指向用户栈 


### 2.7 

> 从 U 态进入 S 态是哪一条指令发生的？

`ecall`请求系统调用, 或者执行非法操作/错误操作, 总之一切能 Trap 的操作.
