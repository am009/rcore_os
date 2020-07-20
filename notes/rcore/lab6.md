# lab6 用户程序, 系统调用, 文件描述符

## 步骤
1. 完全构建新的crate作为user程序的框架
2. 在外面新建makefile文件


## 用户进程
类似于自己做一些rust标准库的事情. 


## 相关的问题

inline asm 中指明memory对编译器有什么影响??

read/write中的syscall调用方式中的类型转换
```
buffer: &mut [u8]
buffer as *const [u8] as *const u8 as usize
```
这是什么意思??