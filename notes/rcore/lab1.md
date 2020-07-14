# lab1中断 rcore 学习笔记


回顾ucore, ucore的lab1也主要讲了中断, lab2讲分页
ucore的进程分了好几个lab, 内核进程, 用户进程, 进程调度.

添加了interrupt/context.rs, 子文件也是一个mod, interrupt文件夹也是一个新的mod, 现在rust2018, 既可以采用src/interrupt/mod.rs, 也可以用src/interrupt.rs来代表整个文件夹.

另外使用了std中总是会panic的unreachable宏.

1. risc-v的异常, trap, 硬件中断的区别和辨别???


### CSR 是什么

Control and status registers
处理特权相关的寄存器

CSR 寄存器（Chapter 4，p59）
https://content.riscv.org/wp-content/uploads/2017/05/riscv-privileged-v1.10.pdf

CSR 指令（Section 2.8，p33）
https://content.riscv.org/wp-content/uploads/2017/05/riscv-spec-v2.2.pdf

操作这相关的寄存器的包装, riscv, 文档在这
https://docs.rs/riscv/0.6.0/riscv/register/index.html
dependencies里写的居然是rcore自己的fork, 而且比官方的多了特别多的commit, 太神奇了, luojia他已经参与进去了两个commit, 也不知道他是怎么参与进去的, 我也想参与进去啊😂, 也许我水平不够吧.

### RISC-V中断
比较关键的一点是sbi做什么, 而操作系统做什么


### 中断准备
中断寄存器stvec指向的是中断的入口. 不像x86有一长条的中断向量表. 有两种模式, 向量模式和直接模式, 直接模式用一个地址处理所有中断, 向量模式则会根据异常的不同跳转到不同的位置. 
mtvec [0]设置为 1可启用此功能, 根据 中断原因 x将PC设置为（ mtval-1 + 4x），而不是通常的mtvec(??)。 这时就应该要在mtvec附近位置放好跳转指令了?? 这样就不用检查mcause从而提高速度了.

### 中断发生
中断随时可能来, 发生的时候, 可能程序指向到一半, 即使是一些临时寄存器也可能正在使用. 因此不能破坏任何现场.

sscratch在用户态用来保存内核栈的地址, 内核态是0, 因为进入内核态的时候os把它清零.
可以利用sscratch判断中断前是内核态还是用户态

当发生中断的时候, 硬件只是会填sepc, scause, stval. 不像x86会保存到栈上. 保存到栈上全靠软件.

如果是用户态, (sscratch切换栈是软件做的??), 再保存现场????????

而我们lab1还是一直内核态, 内核态发生中断, 就保存各种寄存器, 到栈上, 得到Context结构体. Context很简单, 32个通用寄存器, 加上sstatus, sepc. (riscv的pc不在32个通用寄存器里.??)
而scause和stval看作临时变量.(嵌套中断会不会发生问题?)

sstatus里的带P(previous)的位会被设置, 因此需要保存sstatus.

-------------

os/src/interrupt.asm 内含中断保存现场__interrupt, 和恢复现场__restore
首先sp减34*8开辟空间, 保存时使用以sp为基地址的栈上偏移量寻址Context成员(类似栈上临时变量), 为了sp(x2)保持不变, 首先保存x1, 然后把sp加34\*8保存到x1, 再保存x1从而保存原来的sp(那用户态这样就保存了内核栈了???), 再依次保存各种寄存器.
恢复的时候最后恢复sp即可.

### 中断结束
恢复现场后, sstatus和sepc也恢复了, sret, 返回的时候将pc设置为sepc. (恢复sstatus的时候会恢复原来的中断状态.)

### 时钟中断
设置了timer, 这是risc-v内置的timer??
打开了sie, 使能了监管者模式的中断. 那么之前的ebreak为什么会成功? 使能的是哪些中断??

S模式不直接控制 时钟
中断 和软件中断，而是
使用 ecall指令请求 M模式设置定时器或代表
它发送处理器间中断。
该软件约定是 监管者 二
进制接口 (Supervisor Binary Interface)的一部
分。

由于没有一个接口来设置固定重复的时间中断间隔，因此我们需要在每一次时钟中断时，设置再下一次的时钟中断。


### 代码梳理
之前lab0完成了最小的启动, 通过sbi接口打印字符
本次lab1的代码主要完成的是中断相关. 接到控制权后转到了entry.asm 将bss段作为栈, 然后调用rust_main. rust_main 作为初始化代码, 刚启动就执行的代码, 自然是调用各种初始化函数. 这里调用中断初始化, 这部分代码单独放到一个mod(文件夹)内了, 也就是interrupt文件夹. 

中断只要直接加载到