## 编程作业

首先按照 libc 标准, 运行到 c 的`_start()`时, 用户栈里应该是
- 栈底
- ...
- argv
- argc

所以, 只需要在创建进程时, 在用户栈里`push`相应的东西即可

考虑`rCore`的运行流程
- `TaskControlBlock::new()`创建`init_proc`并运行
- `init_proc`调用`fork + exec`启动第一个进程`shell`
- `shell`根据命令行, 启动对应程序, 并加上参数

因此, 需要修改以下函数
- `Rust`用户库的`_start()` 
  - 之前的参数是`argc`和`argv`
  - 现在改成一个参数, 即`argc`的地址, 以和`C`对标
  - 增加额外几行代码, 将`argc`的值, 和`argv`的地址, 计算出来
- `TaskControlBlock::new()`
  - 在完成`init_proc`的`PCB`的初始化后, 往用户栈里`push`一个`0`, 表示`argc = 0`
  - `trap_cx.x[10] = user_sp`, 因为`Rust`用户库的`_start()`需要参数来指示`argc`的地址
- `TaskControlBlock::exec()`
  - 改成标准的`argc`和`argv`的入栈方式 
  - `trap_cx.x[2] = user_sp`, 因为`C`用户库的`_start()`会将`sp`设置为第一个参数, 而不像`Rust`直接使用`a0`
- `sys_exec()`:
  - 其返回值会作为`a0`传递传递给`Rust`的用户库
  - 之前返回值是`argc`, 现在需要修改为`trap_cx.x[2]`, 即`user_sp`
- `ch7_user_shell`
  - 在使用`exec(path, args)`时, 第二个参数会带有`path`
  - 如: 运行`42`时, 其会调用`exec("42", ["42"])`
  - 修改: 在调用前, `remove`掉`args`的第一个元素


## 问答题

> elf 文件和 bin 文件有什么区别？

使用`file`命令后, 得到以下响应
```
ch6_file0.elf: ELF 64-bit LSB executable, UCB RISC-V, version 1 (SYSV), statically linked, stripped
ch6_file0.bin: data
```

`ELF`文件
- 是`Linux`所支持的一种二进制文件, 是可以运行的程序
- 含有程序的元数据(比如程序头表), 内核读取后可以知道应该怎么放入内存中运行该程序

`bin`文件
- 不含元数据的二进制文件
- 可以直接在裸机上运行(只要放到正确的内存位置)
