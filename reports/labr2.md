## 编程作业

跟随文档指引, 使得运行测例 `hellostd` 可以输出 `Unsupported syscall_id: 29`

添加了如下 syscall, 使运行测例 `hellostd` 可以正常运行并输出 `hello std`

`sys_ioctl`: 
- 直接返回 0

`sys_writev`: 
- 参数: 
  - `fd`文件描述符, 
  - `iov`: 数组, 元素为结构体`iovec`
  - `vlen`: 数组长度
- 实现:
  - 遍历数组, 从结构体`iovec`中取出字符串地址和长度, 依次调用`sys_write`打印字符串
- 返回: 
  - 总共打印的字符数

`sys_exit_group`
- 调用`sys_exit`


## 问答题

> 查询标志位定义。
> 
> 标准的 `waitpid` 调用的结构是 
> 
> pid_t waitpid(pid_t pid, int *_Nullable wstatus, int options);
> 
> 其中的 `options` 参数分别有哪些可能（只要列出不需要解释）, 用 int 的 32 个 bit 如何表示?

`options`可能的参数及其表示如下:
- `WCONTINUED`: 8(0b1000)
- `WNOHANG`: 1(0b1)
- `WUNTRACED`: 2(0b10)

