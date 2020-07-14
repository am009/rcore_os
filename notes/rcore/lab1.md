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
https://github.com/riscv/riscv-sbi-doc/blob/master/riscv-sbi.adoc
看这个文档. 
重要的有: 时钟中断要通过sbi设置
异常委托机制的使用: 默认是所有中断都到m模式, mideleg寄存器可以设置把哪些中断委托给s模式. medeleg设置把哪些异常委托给s模式.
sstatus的SIE位是总开关, 中断时会被放到SPIE(sret的时候会恢复回去(riscv-privileged-3.1.10)), 之前的权限模式被放到sstatus的SPP


每个单独的中断屏蔽位在sie寄存器. 当前准备处理的中断位在sip寄存器中, 这两个寄存器本质上都是屏蔽没有区别?? 
只有被委派的对应位能够修改.
所以具体哪些中断被委派了?? 分析一下启动时候opensbi打印的委派寄存器的值吧.


中断寄存器stvec指向的是中断的入口. 不像x86有一长条的中断向量表. 有两种模式, 向量模式和直接模式, 直接模式用一个地址处理所有中断, 向量模式则会根据异常的不同跳转到不同的位置. 
mtvec[0](vec寄存器的指令对齐使得最低位无效)设置为 1可启用此功能, 根据 中断原因 x将PC设置为（ mtval-1 + 4x），而不是通常的mtvec(??)。 这时就应该要在mtvec附近位置放好跳转指令了?? 这样就不用检查mcause从而提高速度了.
stvec同理??

### 中断准备
那Sstatus寄存器的SIE位负责的是中断总开关, 而类似ebreak这样的是异常, 所以不设置这个位也能进行中断.
当用到时钟中断的时候就要设置SIE位打开总开关了.

### 中断发生
中断随时可能来, 发生的时候, 可能程序指向到一半, 即使是一些临时寄存器也可能正在使用. 因此不能破坏任何现场.

sscratch是一个单纯用来存数据的寄存器??
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

S模式不直接控制 时钟中断 和软件中断，而是使用 ecall指令请求 M模式设置定时器或代表它发送处理器间中断。该软件约定是 监管者 二进制接口 (Supervisor Binary Interface)的一部分。

由于没有一个接口来设置固定重复的时间中断间隔，因此我们需要在每一次时钟中断时，设置再下一次的时钟中断。

此外, 由于文档不是特别完善, 需要自己加上各种use语句, 在interrupt中加入timer的init.

### 代码梳理
之前lab0完成了最小的启动, 通过sbi接口打印字符
本次lab1的代码主要完成的是中断相关. 接到控制权后转到了entry.asm 将bss段作为栈, 然后调用rust_main.
正是因为rust_main被汇编调用, 因此声明的时候要加上extern "C"从而使用C的abi.
rust_main 作为初始化代码, 刚启动就执行的代码, 自然是调用各种初始化函数. 这里调用中断初始化, 这部分代码单独放到一个mod(文件夹)内了, 也就是interrupt文件夹. 

进入interrupt模块的初始化函数, 分别是handler的初始化和timer的初始化. 
handler的主要任务, 把准备好的中断处理函数加载到中断向量寄存器stvec, 就可以了???
具体来说, 使用global_asm宏引入了interrupt.asm, 在要用到的地方用extern "C" 声明函数, 最后使用write写入__interrupt地址到stvec.这里中断写入也放到unsafe内的原因是这个函数声明的作用域仅限unsafe作用域内.
__interrupt函数, 将栈上的Context地址放到a0, 把scause放到a1, 把stval放到a2, 最后jal(jump and link) 实现跳转. 因为函数调用约定就是用的jal调用函数, ret返回.
陈渝似乎觉得这里的汇编指令太长, 我倒觉得不长, 就像是把32个寄存器点个名熟悉一下, 毕竟它们本身就挺重要的, 而且寄存器的数量也不会随便拓展.

handle_interrupt函数直接根据cause来调用不同的函数处理. 如果是断点异常, 就打印出来, 将PC加2(看来使用了C拓展减少了指令长度), 时钟中断就调用tick函数, 默认就调用fault函数panic并且打印未解决的异常.
lab1的时候这里context保存无法用Debug trait打印, 我就去掉了panic函数对context的打印.

opensbi估计已经开启了并且委派好了中断?? 

时钟中断的初始化就是首先修改sie寄存器, 允许时钟中断, 再开启sstauts的中断允许位. 再设置第一次的时钟. 每次时钟中断的时候, 都会从中断处理程序那走一遭, 然后调用tick函数计数并继续设置下一次时钟.
目前设置的是每10 0000条指令产生一次时钟中断