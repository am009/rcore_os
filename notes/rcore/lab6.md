# lab6 用户程序, 系统调用, 文件描述符

## 步骤
1. 完全构建新的crate作为user程序的框架, 这里也要新建Makefile文件, 并且可以单独build
2. 在外面新建makefile文件, 依次make两个子目录
3. 中断处理中, 初始化时开启外部中断, 用户态的ecall异常时调用kernel模块的syscall_handler, 增加外部中断处理键盘输入
4. 为线程增加打开的文件descriptor数组, 初始化创建的时候就放入STDIN, STDOUT.
5. fs中增加stdin和stdout全局变量
4. 增加mod kernel
    1. condvar.rs 利用线程的睡眠实现条件变量
    2. syscall.rs 系统调用的总入口.
    3. process.rs 处理线程退出的系统调用
    4. fs.rs 处理文件读取相关的系统调用

## 用户进程

类似于自己做一些rust标准库的事情. 
首先是实现了ecall的包装,从而实现了sys_read, sys_write, sys_exit. 利用sys_write实现了print, println宏. 实现了对用户程序的输入输出的支持.

hello_world主要使用了输出.
nodebook可以把输入的字符回显.

### fs/stdout.rs stdin.rs

让标准输入和输出实现和文件一样的接口(INode)进行读写.

stdout没想到就是一个空结构体
```rust
pub struct Stdout;
```
然后直接实现INode的方法, read和poll都返回不支持的错误, write不允许offset为非0.

标准输入stdin同理, 只允许offset为0, buf中没有值则等待一次条件变量, 否则进入读过程, 要么是stdin的buffer空了, 要么是buf不够长, 返回.

### 系统调用的实现

syscall_handler函数根据传入的系统调用号调用各个子函数, 重要的是子函数的返回值还代表了对当前进程的处理方式.

write: 根据fd在进程的descriptor内获取inode, 调用inode的write_at, 直接返回Proceed和返回值.

read: 调用inode的read_at, 然后根据返回值包装一下. 和write不同的地方在于, 如果返回值为0则park当前线程(阻塞), 此时已经在read_at内部等待了condvar, 调用等待时会把当前线程放入等待队列并sleep_current_thread. 之后syscall_handler在处理返回值的时候发现是Park类型会再切换线程.
直到之后external interrupt键盘输入->push到stdin中->条件变量notify->进程恢复调度.

### condvar
这里的条件变量利用的是线程的休眠, 等待条件变量时进入条件变量内部的队列, 线程休眠. 当notify时则唤醒进程.
只在fs/stdin.rs中被实例化并使用.

如果有多个线程同时等待标准输入的话, 因为现在的线程真的是

## 相关的问题

inline asm 中指明memory对编译器有什么影响??

read/write中的syscall调用方式中的类型转换
```
buffer: &mut [u8]
buffer as *const [u8] as *const u8 as usize
```
这是什么意思??

opensbi开启外部中断往外写的魔数是什么??哪里规定的??

### 其他知识点

cargo项目中除了main.rs是默认的可执行文件, 其他bin文件夹下的文件也会被编译为可执行文件

