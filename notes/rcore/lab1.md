# lab1中断 rcore 学习笔记


回顾ucore, ucore的lab1也主要讲了中断, lab2讲分页
ucore的进程管理分了好几个lab, 内核进程, 用户进程, 进程调度.

添加了interrupt/context.rs, 子文件也是一个mod, interrupt文件夹也是一个新的mod, 现在rust2018, 既可以采用src/interrupt/mod.rs, 也可以用src/interrupt.rs来代表整个文件夹.

由于中断说起来比较顺口, 因此下文中部分地方说中断这个词的时候其实既包括中断又包括异常, 也就是包括那些会跳转到trap vector的事件.

### CSR 是什么

Control and status registers
处理特权相关的寄存器

CSR 寄存器（Chapter 4，p59）
https://content.riscv.org/wp-content/uploads/2017/05/riscv-privileged-v1.10.pdf

CSR 指令（Section 2.8，p33）
https://content.riscv.org/wp-content/uploads/2017/05/riscv-spec-v2.2.pdf

操作这相关的寄存器的包装, riscv, 文档在这
https://docs.rs/riscv/0.6.0/riscv/register/index.html
dependencies里写的居然是rcore自己的fork, 而且比官方的多了特别多的commit, 太神奇了, luojia他已经参与进去了两个commit, 也不知道他是怎么参与进去的, 可能因为他本来就是rust-embedded那边来的吧.
我也想参与进去😂, 也许我水平不够吧.

## RISC-V中断
比较关键的一点是sbi做什么, 而操作系统做什么
https://github.com/riscv/riscv-sbi-doc/blob/master/riscv-sbi.adoc
看这个文档. 
一个稍微比较重要的理念就是supervisor态和user态都可能是虚拟化的, 只有m态不是虚拟化的. 因此一些和虚拟化相关的操作都不能直接从s态掌控. 包括时钟和ipc. 硬件线程间的通信.

### 异常/中断委托

委托机制的使用: 默认是所有中断和异常都转到m模式的mtvec, 通过设置mideleg/medeleg寄存器可以设置把哪些中断和异常委托给s模式. 
下面这段话是privileged isa手册中说的, 意思是m模式产生的ecall异常无法被委托, s模式下的ecall和m模式下的ecall在sedeleg中无法被委托给user模式
```
medeleg[11] and sedeleg[11:9] are all hardwired to zero
Some exceptions cannot occur at less privileged modes
```
所以在tutorial中具体哪些中断被委派了? 启动时候opensbi会打印委派寄存器的值如下.
MIDELEG : 0x0000000000000222
MEDELEG : 0x000000000000b109
mideleg的bit为分布和mip和mie相同, medeleg的bit分布对应那张异常的编号的表.
分析一下得知:
地址对齐异常, 断点异常, 用户模式的ecall, 三种页异常(读写和指令) 被委派.
中断有: s态软件中断, s态时间中断, s态外部中断 被委派

TODO 为什么user态的三个中断没有被委派?? user态中断是m态先处理还是s态先处理??

### 中断的屏蔽
sstatus的SIE位是总开关, 

每个单独的中断也可以针对性地屏蔽, 在sie寄存器有对应的屏蔽位. sie和sip这两个寄存器中, 只有被m态委派的对应位能够修改.

### 中断向量

中断寄存器stvec指向的是中断的入口. 不像x86有一长条的中断向量表. 有两种模式, 向量模式和直接模式, 直接模式用一个地址处理所有中断和异常, 向量模式则会让不同中断跳转到不同的位置, 异常还是直接跳转到Base地址. 
mtvec[0](vec寄存器的指令对齐使得最低位无效)设置为1可启用向量模式中断, 根据中断原因x将PC设置为(base + 4x), 也就是跳转到不同的地址.
<!--这时就应该要在mtvec附近位置放好跳转指令了?? 这样就不用检查mcause从而提高速度了.
stvec同理?? -->
向量模式的用处??TODO 能区分特权级出来吗?? 中断嵌套?? 
如果操作系统写好了不会产生异常, 那么异常肯定是用户态的(除了S态ecall被委托的情况). 只有时钟/软件/外部中断可能会出现在S态嵌套?? 为什么外部中断不都是m态的(虚拟化?)

产生中断时中断时该位的值会被放到SPIE, 之前的权限模式被放到sstatus的SPP.

执行相关的sRet指令的时候, (是不是完全相当于??TODO)类似于产生异常的逆过程. 1是会把sepc恢复到pc, 2是sstatus中各种previous位都恢复到原来的位置. mret, uret类似.

## 中断过程梳理

之前lab0完成了最小的启动, 通过sbi接口打印字符
本次lab1的代码主要完成的是中断相关. 接到控制权后转到了entry.asm 将bss段作为栈, 然后调用rust_main.

### 中断准备

正是因为rust_main被汇编调用, 因此声明的时候要加上extern "C"从而使用C的abi.
rust_main 作为初始化代码, 刚启动就执行的代码, 自然是调用各种初始化函数. 这里调用中断初始化, 这部分代码单独放到一个mod内了, 也就是interrupt文件夹. 

具体来说, 使用global_asm宏引入了interrupt.asm, 在要用到的地方用extern "C" 声明函数, 最后使用write写入__interrupt地址到stvec.这里把写入stvec寄存器也放到unsafe内的原因是这个函数声明的作用域仅限unsafe作用域内.

main函数中调用interrupt模块的初始化, 完成中断的准备. 中断的初始化主要做两件事, 把准备好的中断处理函数加载到trap vector寄存器stvec, 和打开中断总开关和对应的分开关.

进入interrupt模块的初始化函数, 分别是handler的初始化和timer的初始化.handler的初始化函数设置了stvec寄存器为interrupt.asm中符号`__interrupt`的地址, 同时开启sie寄存器中s态外部中断的开关 TODO 外部中断有哪些??.
timer的初始化函数则打开了sie中的s态时钟中断, 和中断的总开关, 并且设置了第一次时钟中断.

那Sstatus寄存器的SIE位负责的是中断总开关, 而类似ebreak这样的是异常, 所以不设置这个位也能进入trap处理. 当lab1后半部分用到时钟中断的时候就要设置SIE位打开总开关了.

### 中断发生
中断随时可能来, 发生的时候, 可能程序执行到一半, 即使是一些临时寄存器也可能正在使用. 因此不能破坏任何现场.

断点异常和S态时钟中断都被opensbi在deleg系列寄存器中委托过了, 因此这两个中断产生时就会转到我们S态的中断向量处.

当有中断或者异常发生的时候就会跳转到之前设置好的`__interrupt`处, 硬件只是会修改sepc, scause, stval等寄存器的值, 而不像x86会直接保存到栈上. 保存到栈上全靠我们操作系统的指令. 而`__interrupt`主要做的就是保存现场并恢复. 
首先是把各个寄存器压栈形成Context, Context的结构并不复杂, 32个通用寄存器, 加上sstatus, sepc. (riscv的pc不在32个通用寄存器里)
然后将栈上的Context地址放到a0, 把scause放到a1, 把stval放到a2, 最后jal(jump and link) 实现跳转. 因为函数调用约定就是用的jal调用函数, ret返回. link代表把下一个指令的地址放到link寄存器中.
当handle_interrupt函数返回的时候, 就回到了汇编代码interrupt.asm中, 到了__restore这个部分, 自动开始了恢复中断的过程.

sscratch是一个单纯用来存数据的寄存器, 在tutorial中, sscratch在用户态用来保存内核栈的地址, 内核态是0, 因为进入内核态(进入中断)的时候os把它清零. 
之后为了支持用户态程序, 就需要用到sscratch, 先切换栈再保存Context, 而我们lab1还是一直内核态, 内核态发生中断, 就可以直接保存各种寄存器在当前栈上, 取出栈上的指针作为Context结构体的借用传入interrupt_handler.
sstatus里的带P(previous)的位会被设置好, 因此需要保存sstatus. 而scause和stval就直接看作局部于这次中断处理的临时变量(handle_interrupt的参数), 不保存, 在中断处理的过程中用寄存器传递.

os/src/interrupt.asm 内含中断保存现场__interrupt, 和恢复现场__restore
首先sp减34*8开辟空间, 保存时使用以sp为基地址的栈上偏移量寻址Context成员(类似栈上临时变量), 为了sp(x2)保持不变, 首先保存x1, 然后利用空闲出来的x1去计算原来的sp, 也就是把sp加34\*8保存到x1, 再保存x1(作为sp(x2)), 再依次保存各种寄存器.
恢复的时候最后恢复sp即可.

handle_interrupt函数直接根据cause来调用不同的函数处理. 如果是断点异常, 就打印出来, 将PC加2(看来使用了C拓展减少了指令长度), 时钟中断就调用tick函数, 默认就调用fault函数panic并且打印未解决的异常.

#### 时钟中断

现在RISC-V的timer一般都是内置在cpu内的, 不像x86是通过外部芯片产生时钟中断.

每次时钟中断的时候, 都会从中断处理程序那走一遭, 然后调用tick函数计数并继续设置下一次时钟. 目前设置的是每10 0000条指令产生一次时钟中断

> S模式不直接控制 时钟中断 和软件中断，而是使用 ecall指令请求 M模式设置定时器或代表它发送处理器间中断。该软件约定是监管者二进制接口 (Supervisor Binary Interface)的一部分。

上面这句话来自那本中文的《riscv手册》. 虽然时钟的设置是通过sbi接口, 也就是ecall指令去使用m态程序(opensbi)提供的服务, 但时间到了的通知, 还是通过S态时钟中断. (猜测是opensbi设置时钟, 得到m态的时钟中断信号的时候, 传递下来, 产生S态的时钟中断信号.)

由于没有一个接口来设置固定重复的时间中断间隔，因此我们需要在每一次时钟中断时，设置再下一次的时钟中断.


#### 断点异常

ebreak指令会产生断点异常. 无论是ebreak还是ecall, 产生异常时的sepc都是指向该指令, 而不是下一条指令.

??那其他异常呢? 是下一条指令吗?? TODO

### 中断结束

当handle_interrupt函数返回的时候, 返回到调用它的interrupt.asm中的jal指令之后, 开始恢复之前保存的现场. 直接把各个保存的寄存器恢复, 这样寄存器的状态就是发生中断时的状态. 恢复现场后, sstatus和sepc也恢复了, sret, 返回的时候将pc设置为sepc. 并且恢复sstatus寄存器, 把里面的previous位都还原. 如果中断之前是打开中断的状态, sret后也会回到打开中断状态. 最终恢复到中断前被打断的位置继续执行。

## 其他问题

1. risc-v的trap, 异常, 中断的区别和辨别?
会导致进入trap vector中断处理程序的有中断和异常, 这根据mcause的最高位可以判断, 各种原因分别有自己的编号.
vector模式的中断处理中, 如果是异常, 那么还是跳转到Base地址, 如果是中断, 则会跳转到base地址\+4\*中断原因编号.
而这里中断的分类首先是按照特权级, 其次是被分为硬件中断, 软件中断和时钟中断. 
不同特权级的ecall指令则被分类在Exception中. 

unreachable宏: 在rust的core中, 总是会panic, 并且为编译器的优化提供方便

sscratch 是不是忘了清零?

handle_interrupt被interrupt.asm调用, 要不要加extern "C"?

可以利用sscratch判断中断前是内核态还是用户态?

## interrupt pending 寄存器

machine/supervisor/user interrupt pending寄存器是提供有关正在等待的中断的信息.

TODO 没有完全彻底懂, 不过rcore-tutorial没怎么用到, 就等下次再学吧, 有点难.

### 背景

当多个中断发生的时候, riscv首先处理特权级最高的, 特权级相同的时候按照 外部->软件->时钟的顺序处理(使得最差情况时的处理时间最小). 因此当中断同时发生的时候, 或者ISA中断服务例程执行的时候, 其他中断却来了, 此时产生了中断pending. 
让当前的中断例程能感知到新中断的存在, 在arm架构中好像就有相关的应用. 看到一个是应用是省略相继中断间的重新弹栈压栈, 提升性能.

### 作用

当从mip(xip)寄存器中获取值的时候, 得到的是对应寄存器和对应中断产生信号的OR之后的值. 也就是如果这个中断真正在等待, 对应的pending位就为1.

高特权级如果设置了低特权级的对应位, (是回到对应特权级的时候还是立刻?)就会产生对应的中断. 如, m态的程序(设置好对应的stval, scause??), 就可以通过设置mip对应的supervisor的中断pending位, 来伪装触发中断. m态的对应pending位是只读的, mip中对应低特权级的pending位则可读, 也可以写触发上述效果.
S态软件中断, U态软件中断(基本上)靠这种方式产生. 
s态和u态的外部中断有什么例子?? u态的外部中断可以有中断处理器产生.

有没有办法产生u态ecall?? 似乎不能... 确实s态ecall需要被产生

软件模拟硬件中断?? 也就是设置各种previous位, 和sval, scause再自己跳转到中断处理里去.

手册里说只有开启了mstatus中总的中断enable, 对应中断的pending位和enable位都开启才会产生中断?? 这是为什么?? TODO
1. 可能指的是产生中断的时候, 取mip寄存器逻辑上对应pending位会被设置
2. 可能真的需要手动设置对应pending位都为1?? 有点离谱, 这是mie寄存器的工作吧, 这样会触发中断吧



