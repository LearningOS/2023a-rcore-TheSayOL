## 编程作业

### 3.1

```rust
#[allow(dead_code)]
pub const SDCARD_TESTCASES: &[&str] = &[
    "busybox touch abc",
    "busybox mv abc bin/",
    "busybox ls bin/abc",
];
```
> 其中不能正常工作的命令是 busybox mv abc bin/。busybox touch 用于创建文件 abc，busybox ls bin/abc 用于检查文件 bin/abc 是否存在。你的任务是修改代码，使得 busybox ls bin/abc 正常输出 bin/abc。


使用`strace busybox mv abc bin/`, 发现
- 进行了类似的系统调用`fstat('bin/', ...)`, 返回值为`0`
- 进行了类似的系统调用`fstat('bin/abc', ...)`, 返回值为`-1, ENOENT`
- 进行了系统调用`rename()`

猜想流程为
- 用`fstat()`获取文件状态, 通过返回值, 判断文件是否存在
- 使用`rename()`修改文件

修改代码, 执行内核, 发生错误, 观察错误提示, 发现在`mv`的过程中, 
- 进行了系统调用`fstatat('bin/abc', ...)`, 返回值`-64`
- 内核输出: `mv: can't stat file, No error infomation`

显然此处发生错误是应该的, 但是`No error info`是不应该的, 观察内核系统调用
- 调用`sys_fstatat()`
- 调用`get_stat_in_fs()`, 返回`Err(SyscallError::ENONET)`
- `ENONET`的值是`-64`, 但是通过网上搜索, 发现`ENOENT`应该是`-2`
- 仔细观察, 才发现原来是写错了, 改正就正常了

在`fstatat()`通过之后, 内核继续运行, 试图`sys_renameat2()`, 未实现
- 增加代码, 进行了简单的实现, 即, 直接调用`sys_rename()` 

运行正常

### 3.2

```rust
#[allow(dead_code)]
pub const SDCARD_TESTCASES: &[&str] = &[
    "busybox touch def",
    "busybox mv def bin",
    "busybox ls bin/def",
];
```
> 其中不能正常工作的命令是 busybox mv def bin。它和实验3.1的区别在于，bin 目录后没有斜线 /。你的任务是修改代码，使得 busybox mv def bin 正常输出 bin/def。

修改了`sys_renameat2()`的逻辑
- 增加一条判断语句: 如果`new_path`是个目录并且后面没跟下划线, 那么给他补上


## 问答题

### 1.1

> 部分往届内核及运行指引 提到了 cargo 的离线编译与缓存。Rust 库具体会被 cargo 缓存到哪里呢？
> 
> 我们可以通过跳转或者搜索，去看 log 库的代码：
>
> 思考题1.1：这些代码具体在 ~/.cargo 下的哪个文件夹？

`log`库保存于`~/.cargo/registry/src/index.crates.io-6f17d22bba15001f/log-0.4.19/src/lib.rs`


### 2

> 思考题2：在部分往届内核及运行指引 一节提到的内核中挑选一个，描述它在默认情况下启动后会执行哪些测例（抑或是直接启动终端）。你不一定要真的运行那个内核，读文档或搜索即可。

`umi`
- 在完成初始化后, 默认执行如下三个测试
    ```rust
    mem::test_phys().await;
    fs::test_file().await;
    self::test::test_all().await;
    ```
- 其中`test_all()`的第一个测例是`time-test`




### 3.1 


> 思考题3.1：为什么要在开头结尾各输出一句，会不会太过重复？（提示：考虑执行出错的情况，或者 sys_exit ）

仅仅开头输出的一句, 只能获取 sys_call_id 和参数

而结尾加的一句可以获取返回值, 而返回值很多时候都很有用



PS: 文档中给出的代码可能有误, 如下, 在本机上无法运行, `println`应该在返回之前
```rust
#[no_mangle]
pub fn syscall(syscall_id: usize, args: [usize; 6]) -> isize {
    println!("[syscall] id = {}, args = {:?}, entry", syscall_id, args);
    #[cfg(feature = "futex")]
    syscall_task::check_dead_wait();
    ......
    let ans = deal_result(ans);
    ans
    println!("[syscall] id = {}, args = {:?}, return {}", syscall_id, args, ans);
}
```



### 3.2

> 思考题3.2：为什么要结尾还要输出一遍 syscall 的完整参数，只输出返回值行不行？（提示：考虑像 sys_yield 这样的 syscall）

考虑调用`sys_yield()`的情况
- 线程 A 调用`sys_yield()`并没有参数
- 随后切换到另一个线程 B, 其可能是在一个系统调用中阻塞的
- 随后 B 从系统调用中返回, 返回值和参数应该是 B 的
- 如果只输出返回值, 就无法判断是谁的返回值

