# rcore学习笔记

这是我这几天学习rcore-Tutorial第三版时的笔记, 汇总到了一起. 可能会写得比较乱?

[lab1](#lab1中断) 学习了不少RISC-V的中断相关的基础知识, 之后补的中断相关的知识也补在这里了
[lab3](#lab3虚拟内存管理), [lab4](#lab4线程与调度) 对实现细节写得详细一些, 稍微看懂一点代码就写上去了, 很多函数的实现细节都写下来了.

[实验题](#实验题)在最后, 目前完成了lab4上和lab6的实验题, lab4下做到一半.

## lab1中断

回顾ucore, ucore的lab1也主要讲了中断, lab2讲分页
ucore的进程管理分了好几个lab, 内核进程, 用户进程, 进程调度.

添加了interrupt/context.rs, 文件也是一个mod, interrupt文件夹也是一个新的mod, 现在rust2018, 既可以采用src/interrupt/mod.rs, 也可以用src/interrupt.rs来代表整个文件夹作为mod.

由于中断说起来比较顺口, 因此下文中部分地方说中断这个词的时候其实既包括中断又包括异常, 也就是包括那些会跳转到trap vector的事件.

### CSR 是什么

Control and status registers, 大部分是处理特权相关的寄存器. 为操作系统程序提供特权, 方便管理用户态程序.

操作这相关的寄存器的包装, riscv这个crate, 这是相关的文档. 
https://docs.rs/riscv/0.6.0/riscv/register/index.html
dependencies里写的居然是rcore自己的fork, 而且比官方的多了特别多的commit, 太神奇了. 因此可能有我们rcore自己的fork实现的东西, 而文档里没有.

### RISC-V中断
比较关键的一点是sbi做什么, 而操作系统做什么
https://github.com/riscv/riscv-sbi-doc/blob/master/riscv-sbi.adoc
可以看看上面这个文档. 
一个稍微比较重要的理念是supervisor态和user态都可能是虚拟化的, 只有m态不是虚拟化的. 因此一些对虚拟化有用的操作都不能直接从s态掌控. 包括时钟和ipc. 硬件线程间的通信.

#### 异常/中断委托

委托机制的使用: 默认是所有中断和异常都转到m模式的mtvec, 通过设置mideleg/medeleg寄存器可以设置把哪些中断和异常委托给s模式.

下面这段话是privileged isa手册中说的
> Some exceptions cannot occur at less privileged modes, and corresponding x edeleg bits should be hardwired to zero. In particular, `medeleg[11]` and `sedeleg[11:9]` are all hardwired to zero.

最后一句的意思是m模式产生的ecall异常无法被委托, s模式下的ecall和m模式下的ecall在sedeleg中无法被委托给user模式.

所以在tutorial中具体哪些中断被委派了? 启动的时候opensbi会打印委派寄存器的值如下.
MIDELEG : 0x0000000000000222
MEDELEG : 0x000000000000b109
mideleg的bit为分布和mip和mie相同, medeleg的bit分布对应那张异常的编号的表.
分析一下得知:
地址对齐异常, 断点异常, 用户模式的ecall, 三种页异常(读写和指令) 被委派.
中断有: s态软件中断, s态时间中断, s态外部中断 被委派

#### 中断的屏蔽

sstatus的SIE位是总开关, 每个单独的中断也可以针对性地屏蔽, 在sie寄存器有对应的屏蔽位. sie和sip这两个寄存器中, 只有被m态委派的对应位能够修改.

#### 中断向量

中断寄存器stvec指向的是中断的入口. 不像x86有一长条的中断向量表. 有两种模式, 向量模式和直接模式, 直接模式用一个地址处理所有中断和异常, 向量模式则会让不同中断跳转到不同的位置, 在基地址的基础上加上一定的偏移量, 而异常还是直接跳转到基地址. 
mtvec[0](vec寄存器的指令对齐使得最低位无效)设置为1可启用向量模式中断, 根据中断原因x将PC设置为(base + 4x), 也就是跳转到不同的地址.

产生中断时各种中断相关的位会被放到对应的previous位中, 之前的权限模式被放到sstatus的SPP.

执行相关的sRet指令的时候, 类似于产生异常的逆过程. 1是会把sepc恢复到pc, 2是sstatus中各种previous位都恢复到原来的位置. mret, uret类似.

### 中断过程梳理

之前lab0完成了最小的启动, 通过sbi接口打印字符
本次lab1的代码主要完成的是中断相关. 接到控制权后转到了entry.asm 将bss段作为栈, 然后调用rust_main.

#### 中断准备

正是因为rust_main被汇编调用, 因此声明的时候要加上extern "C"从而使用C的abi.
rust_main 作为初始化代码, 刚启动就执行的代码, 自然是调用各种初始化函数. 这里调用中断初始化, 这部分代码单独放到一个mod内了, 也就是interrupt文件夹. 

具体来说, 使用global_asm宏引入了interrupt.asm, 在要用到的地方用extern "C" 声明函数, 最后使用write写入__interrupt地址到stvec.这里把写入stvec寄存器也放到unsafe内的原因是这个函数声明的作用域仅限unsafe作用域内.

main函数中调用interrupt模块的初始化, 完成中断的准备. 中断的初始化主要做两件事, 把准备好的中断处理函数加载到trap vector寄存器stvec, 和打开中断总开关和对应的分开关.

进入interrupt模块的初始化函数, 分别是handler的初始化和timer的初始化.handler的初始化函数设置了stvec寄存器为interrupt.asm中符号`__interrupt`的地址, 同时开启sie寄存器中s态外部中断的开关
timer的初始化函数则打开了sie中的s态时钟中断, 和中断的总开关, 并且设置了第一次时钟中断.

那Sstatus寄存器的SIE位负责的是中断总开关, 而类似ebreak这样的是异常, 所以不设置这个位也能进入trap处理. 当lab1后半部分用到时钟中断的时候就要设置SIE位打开总开关了.

#### 中断发生
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

#### 中断结束

当handle_interrupt函数返回的时候, 返回到调用它的interrupt.asm中的jal指令之后, 开始恢复之前保存的现场. 直接把各个保存的寄存器恢复, 这样寄存器的状态就是发生中断时的状态. 恢复现场后, sstatus和sepc也恢复了, sret, 返回的时候将pc设置为sepc. 并且恢复sstatus寄存器, 把里面的previous位都还原. 如果中断之前是打开中断的状态, sret后也会回到打开中断状态. 最终恢复到中断前被打断的位置继续执行。

### interrupt pending 寄存器

machine/supervisor/user interrupt pending寄存器是提供有关正在等待的中断的信息.

这里我也没有彻底学懂, 不过rcore-tutorial没怎么用到.

#### 背景

当多个中断发生的时候, riscv首先处理特权级最高的, 特权级相同的时候按照 外部->软件->时钟的顺序处理(使得最差情况时的处理时间最小). 因此当很多中断同时发生的时候, 或者ISA中断服务例程执行的时候, 其他中断却来了, 此时产生了中断pending. 
让当前的中断例程能感知到新中断的存在有一定的作用, 我临时搜索了一下发现, 在arm架构中好像就有相关的应用. 看到一个是应用是省略相继产生的中断间的重新弹栈压栈, 提升性能.

#### 作用

当从mip(xip)寄存器中获取值的时候, 得到的是对应寄存器和对应中断产生信号的OR之后的值. 也就是如果这个中断真正在等待, 对应的pending位就为1.

高特权级如果设置了低特权级的对应中断的pending位, (不知道是回到对应特权级的时候还是立刻?)就会产生对应的中断. 如, m态的程序就可以通过设置mip对应的supervisor的中断pending位, 从而让低特权级的程序产生中断. 各种m态的中断的pending位在mip寄存器中是只读的, mip中对应低特权级的pending位则既可读, 也可以写触发上述效果.
S态软件中断, U态软件中断(基本上)靠这种方式产生. 


## lab2内存管理

包括临时堆内存管理, 物理内存管理.

### 临时堆内存管理

ucore中是先实现按页的物理内存管理, 再实现的任意大小的管理的. 而这里完全不一样, 先是bss段留了8M空间作为堆, 给操作系统动态内存分配用, 再去单独实现按页的物理内存管理.

这里我暂时使用临时堆内存管理这个新词, 表示为了使用rust提供的一些需要使用堆内存的数据结构而在bss段上划分出一块空间作为堆. rcore-tutorial这里我们直接使用现有的buddy system内存分配算法, 在代码中开辟8M大小的bss段空间(u8数组), 作为被分配的空间.

我们分配算法和rust的对接主要在于Trait GlobalAlloc, 实例化之后用 `#[global_allocator]`标记就可以使用动态内存分配了(可以使用一些需要动态内存分配的内置数据结构, 如Box, Vec等). 接口也是和C语言中malloc/free类似的接口: alloc和dealloc.

#### bss+buddy system实现细节

直接分配u8 static数组,数组名字指向的就是对应的空间.
添加buddy system这个包. spin和lazy_static也顺便加上.
创建memory文件夹作为新的mod, 创建init函数被main调用. 创建一个HEAP全局变量作为分配器, 并在init函数里面把那个数组的内存初始化给它. 
想把数组的名字作为指针, 只需要调用.as_str()然后as转换为usize就可以了.
这样, main函数调用完heap的init之后就可以分配堆空间了.

#### 使用自己的堆分配算法

也可以不使用buddy_system, 答案中的heap2使用自己的algorithm crate的bitmap_vector_allocator提供分配算法支持, 这里自己实现分配算法也可以很简单, 代码量挺少的.

堆分配算法和其他代码的接口一个是`#[global_allocator]`标注, 一个是init函数.
这里使用的是bitmap标记空闲, 以字节为单位, 查找时按照对齐要求的倍数顺序查找(作为内存的开头), 直到遇到了空闲处.
只标记4096字节, 最多只能管理4K的内存. 这里的实现也是对给定内存范围的对应内存的index(偏移)做分配, 每次分配得到的只是一个偏移, 需要去找到对应的内存地址.

不像buddy_system实现好了GlobalAlloc, 为了global_allocator要实现alloc::alloc::GlobalAlloc这个trait. 需要实现分配和回收两个函数, 传入的是core::alloc::Layout, 并且需要处理指针类型 *mut u8. 直接整个实现都是unsafe的. Heap全局变量也不再是简单的直接是一个实例, 而是在VectorAllocatorImpl的基础上包了一层Option, 再包UnsafeCell. UnsafeCell取内部的值需要get再as_mut, Option就直接unwarp, 就可以得到内部的VectorAllocatorImpl调用alloc/dealloc函数.

包一层Option有一个好处就是可以提供默认值. 初始化之前, Option里是None, 初始化函数使用replace函数替换成实例之后才能分配, 否则会在unwrap的时候panic

完成之后, 将main函数堆测试代码的两个循环数量从10000减少到100才能正常通过测试. 这个自己实现的算法毕竟管理的内存比较少.

### 按页的物理内存管理

物理内存管理不像临时堆内存管理只是为了让我们操作系统自己用, 它是虚拟内存管理的基础. 否则的话只要直接把buddy_system的LockedHeap的init函数中传入的内存改成我们可用的所有内存范围, 就能让我们操作系统自己用上这些内存了.

#### 封装地址类型与frame tracker
内存地址空间确实是非常常用的东西. 无论是分页内存管理还是mmio, 之后肯定会大量用到内存地址的. 因此这里封装出了地址类型.
1. 封装地址类和页号类
1. 抽象实现From trait在地址和页号间相互转换
1. 抽象实现地址转页号的向上/下取整
1. 抽象实现和usize的加减输出操作.

还要在memory中新建mod range. 提供对内存地址range的支持. 它在基础的core::ops::range基础上增加相互转换, 和len函数, 和迭代, 重叠检测, 虚拟物理相互转换, 取下标, 包含检测的功能.

frame tracker 作为物理页面的智能指针, 继承PhysicalPageNumber, 实现Drop.
frame模块除了frame_tracker的实现, 同时承载分配相关的实现: allocator.rs 对分配算法进行包装, 对接frame tracker和分配. 分配算法实现Allocator trait(一次分配回收一个元素的index), 包装后提供初始化创建(起始物理页号和页数量), 分配一个页(frame tracker), frame tracker析构的时候自动调用回收.
由于是简单的一次分配一个元素, 而且可以是离散的, 简单地使用一个栈进行分配. 创建StackedAllocator, 在allocator.rs中作为AllocImpl, 就会使用这个算法了.

frame tracker创建的时候不会自动申请页面, 因此想要获得frame tracker需要通过allocator分配, 不能自己构造.

#### 分配哪些内存
可以直接根据qemu内存映射, riscv从0x8000_0000开始到0x8800_0000这128M初始内存, 直接硬编码拿来. 硬编码在 MEMORY_END_ADDRESS, 另外还要设置这些全局变量KERNEL_END_ADDRESS, MEMORY_START_ADDRESS, KERNEL_HEAP_SIZE.

在ucore中, 一般以函数指针结构体作为接口, 让不同的分配算法提供相同的接口. 而且还花大量精力, 使用Page结构体, 链表组织空闲页面.(物理内存管理), 而我们这里实现的就简单得多, 对物理页的下标直接管理.

#### FrameAllocator

实现单页单页的物理内存的分配和回收. 内部使用的算法是StackedAllocator, 非常简单, 一个Vec, 新建的时候把一段物理内存范围输入, 每次分配取栈顶的一页, 每次回收页就压入栈中. 按照单页的分配和回收使得实现起来非常简单, 时间空间复杂度也会很低.
接着在StackedAllocator基础上包装出FrameAllocator, 把对下标的分配转化成真正的内存地址, 并且返回时返回Frame Tracker.

物理内存的分配器目前只实现的单页单页的分配和回收, 这一点我之前其实觉得挺合理的, 因为现在离散式虚拟内存技术已经非常成熟, 我们操作系统用到的现在全都是虚拟地址, 需要"连续的内存"时也一般是需要连续的虚拟地址空间, 因此物理地址的分配完全可以是这样一页一页离散的. 
但是之后的lab可以看到, DMA确实出现了需要连续的物理内存空间这种需求, 看来这里的实现确实值得改进...

### 用到的其他小知识点
pub(super) 指只对父模块是public的
https://doc.rust-lang.org/reference/visibility-and-privacy.html

KERNEL_END_ADDRESS是lazy_static, 因为不用会报错"pointer-to-integer cast" needs an rfc before being allowed inside constants

下面这个impl代表某种类型, 而且最终会被确定下来
https://doc.rust-lang.org/std/keyword.impl.html
```rust
pub fn iter(&self) -> impl Iterator<Item = T> {
    (self.start.into()..self.end.into()).map(T::from)
}
```

下面这句话的下划线似乎代表让编译器推断类型
```rust
let offset = ptr as usize - &HEAP_SPACE as *const _ as usize;
```

## lab3虚拟内存管理

### Sv39页内存管理

Sv39最大支持512G地址空间, 分为3级页表. 每级页表大小都是一页, 因为8B * 512 = 4K. 
最高级的页表, 每一项表示1G的地址空间, 第二级页表每项表示2M地址空间, 最低级的页表每一项表示4K地址空间.
虚拟地址空间64位只有低39位有效, 63-39 位的值必须等于第 38 位的值. 也就是说, 根据最高位是不是1, 512G地址空间被分为低256G(高位都是0), 和高256G(高位都是F).

39位的划分: 页内12位 + 9 + 9 + 9
对应的内存大小: 每页4K, ---(512页)---> 2M -(512)-> 1G -> 512G
十六进制表示: 0x1000 -> 0x20 0000 -> 0x4000 0000 -> 0x80 0000 0000
现在架构中最大可寻址的物理地址有56位. 也就是有56-12=44位标识页
而页表项中`[53-10]`这44位用来标识一个物理页. 也就是物理地址的过高位和低12位去掉之后还要右移两位才可以对应上页表项中. 页表项内低10位自然就是标志位. 页表项最低位(Valid位), 为0则表示该页表项无效.

然而三级和二级页表项不一定要指向下一级页表, 可以作为大页... 如果RWX位全0才是指向下一级页表, 否则作为大页, 项中指向映射的开始页, 向后自动映射2M/1G内存. 这方法厉害啊, 可以在线性映射的时候节约不少内存空间. 另外, 大页也需要按照自己的大小对齐.

satp寄存器指向页表. 要在修改 satp 的指令后面马上使用 sfence.vma 指令刷新整个 TLB。手动修改一个页表项之后可以通过在sfence.vma后面加上一个虚拟地址来刷新单独的页表项中这个虚拟地址的映射.

#### 页表工作方式
1.  首先从 `satp` 中获取页表根节点的页号，找到根页表
2.  对于虚拟地址中每一级 VPN（9 位），在对应的页表中找到对应的页表项
3.  如果对应项 Valid 位为 0，则发生 Page Fault
4.  如果对应项 Readable / Writable 位为 1，则表示这是一个叶子节点。
    页表项中的值便是虚拟地址对应的物理页号
    如果此时还没有达到最低级的页表，说明这是一个大页
5.  将页表项中的页号作为下一级查询目标，查询直到达到最低级的页表，最终得到页号

### 内核启动

#### 内核地址空间的变化

内核的地址空间要抬高, 在512G虚拟地址中不是平移256G. 数据段起始地址变成0xffff ffff 8020 0000, 在Sv39看来是0x7f_8020_0000 而原来是0x00 8020 0000,
多了0x7f 0000 0000, 也就是平移了508G.

另外观察到: 0x80200000 = 2G + 2M

#### 启动过程

1. 0x80200000处(entry.asm)使用汇编开启分页模式
1. 开启分页模式的一瞬间, 当前的PC还在原来的位置, 因此需要映射0x80200000附近的位置, 同时需要映射0xffffffff80200000 附近的位置 直接使用大页映射, 4K空间作为最高级页表, 映射两个1g, 简单方便省内存. (这里之后的lab会为了访问磁盘设备需要再映射1g的页).
1. 跳转到rust内(rust_main)

### 代码变化梳理

内核启动之后, rust_main在之前的初始化的基础上, 我们还会新建一个kernel的MemorySet内存地址映射, 并激活它. 这里的MemorySet更多主要是为了之后的进程的地址空间映射准备的. 之后lab的main函数在初始化的时候就不会再激活这样的映射了.

需要修改的地方有:
1. ld脚本需要修改链接的基址, 这个比较简单
2. 启动的asm文件, 需要加上临时页表, 和装载临时页表的语句
3. 加入虚拟地址结构体, 实现一些相关trait, 增加一个偏移量常量KERNEL_MAP_OFFSET

修改完这几个, 内核依然能够正常运行进入rust_main.
接着修改, 下面这前两步的工作量最大
1. 封装页表项, 页表等
1. 实现MemorySet, 和内部的Mapping和Segment 
2. main函数中新建kernel类型的映射并激活

#### entry.asm
这里只说说这个页表项
```asm
.quad (0x80000 << 10) | 0xcf
```
由于取页号放到第10位开始的位置, 也就相当于0x8000 0000 >>12 <<10, 0xcf表示 VRWXAD这几个标志位均为 1, 表示这个页具有RWX属性.

#### 虚拟地址的封装 memory/address.rs
完善之前物理地址的封装, 加上虚拟地址
1. 指针转换为虚拟地址, 实现这个trait, 这样任何指针类型都可以直接转虚拟地址类型
2. deref_kernel和deref可以用地址类型转换成任意类型的指针, 而且生命周期是static的. pageNumber类型的deref使得可以直接获得页表大小的u8数组.
3. 对VirtualPageNumber类型实现levels函数, 获得三级页号.
4. 也为address类型实现page_offset函数, 取得页内偏移.


#### 实现页表 memory/mapping
不仅封装了页表, 页表项, 还封装了mapping结构, 类似ucore的vma
1. page_table_entry.rs 封装页表项. 实现了Flags类, 表示每个entry低8bit的标志位. 用implement_flags宏抽象标志位的读的实现. 提供了address函数, page_number函数用来找到页表项指向的页面, 实现了flags函数获取flags, 还有is_empty函数, has_next_level函数, 最后实现了Debug trait的打印
2. page_table.rs 封装了页表(页表项数组). 这里还需要把之前的frame_tracker增加了derefMut到u8数组的trait. 封装了PageTableTracker作为PageTable的智能指针. 内部包含一个FrameTracker, 实现自动释放内存的功能. 和PageTable只有一个Deref的距离. 创建页表时要申请物理页将FrameTracker包装为PageTable的智能指针类型再使用. FrameTracker管理一个页, 并且能转换为任何类型, 作为任何类型的智能指针. 而包装了FrameTracker的PageTableTracker则更加具体, 仅仅作为页表的智能指针. 
3. segment.rs 封装了线性映射类型, 实现了遍历某个映射中每个页的功能. 映射有两种类型, 操作系统使用的线性映射, 和按帧分配的离散映射. 后者只能遍历虚拟页, 前者可以直接使用虚拟转物理的转换trait遍历物理页.
4. mapping.rs 负责管理各种页表. 使用vec保存PageTableTracker这个智能指针, 同时另外保存根页表的物理页号(对应页表寄存器). 实现了激活该页表activate函数, new函数新建页表同时分配根目录表, map函数映射一个segment, map_one函数映射一页, unmap移除映射, 会创建页表的find_entry函数(和ucore中那个函数类似), 虚拟地址查找物理地址的lookup函数.
5. memorySet总览全局, 包含了Mapping结构体和Segment数组, 实现添加和删除映射的总接口, 调用下部的mapping的添加和删除映射的接口. 另外还实现了让main函数调用的new_kernel新建memorySet和各种映射的函数, 之后的lab里还要实现读取ELF创建映射的函数

### 实现细节

#### Mapping
mapping负责管理页表, 整个文件100多行, 非常重要. 
1. map_one函数, 映射一个页, 调用find_entry找到对应的entry, 为空则新建并填入Page: `*entry = PageTableEntry::new(ppn, flags);`由于page_table就是page_table_entry数组, 因此直接赋值由于实现了Copy, 就导致页表项写入.
2. lookup函数, 这个函数是静态的!! 首先拿出当前的页表寄存器内的值, 找到页目录表. 把参数的虚拟地址转为页号调用levels函数方便获取每级下标. 然后先取好最高级页表的下标, 再在循环中如果有下一级页表, 不断取下标, 直到页表项为空(此处没有判断valid?), 或者不再有下一level, 此时的entry就保存了base地址, 加上虚拟地址低位的offset(不一定只有12位)得到真正的地址.
3. find_entry函数, 这个函数和lookup有些类似, 但是是从自己的mapping实例的页表物理页号中找到页表, 找的过程中如果页表不存在就直接分配新的页作为页表, 总是能找到页表项, 而且找的总是代表4k一页大小的第三级页表项.
4. unmap函数, 调用find_entry函数并调用clear
5. map函数, map一整个segment. 如果是线性映射, 则遍历虚拟地址不断调用map_one填页表项, 有数据复制数据, 最重要的特点是不用分配物理页面. 如果是离散映射, 则遍历虚拟地址不断分配页面, 把分配到的页面填充0. 拷贝数据的时候映射还没建立, 需要从物理地址加offset这个通用的访问物理内存的映射来复制, 还要考虑区间与整页不对齐的情况, start变量指从页开头开始的偏移, 指向需要复制数据的开始位置. stop变量也是偏移. 每次循环只处理一页. 当开始位置大于当前页的起始位置, 说明是第一页, 需要从开始位置而不是页开头进行复制. 否则就从开始位置复制. 当结束位置减去页起始位置, 小于页的大小的时候, 就说明是最后一页, 需要复制到结束位置为止, 而不是页结束位置.

#### MemorySet

MemorySet 就是一个进程的所有内存空间管理的信息了. 内部包含Mapping, 负责管理页表, 用一个数组保存PageTableTracker(自己管理页表占用的物理页面), 并且另外保存页目录表. 包含segment数组, 内含每个映射, 和allocated_pairs数组, 保存虚拟页号到物理页智能指针(FrameTracker)的二元组, 拿着分配的物理页.
简而言之, MemorySet包含1页表2映射3物理页

添加新的映射的时候, 一方面要添加到页表里去, 一方面要加入映射vec保存, 如果申请了物理页要放到物理页vec中. 还检查是否和当前内存空间重叠.

由于内核换了位置(使用了虚拟地址), 需要在memory/config中加入MMIO 设备段内存区域起始地址: DEVICE_START_ADDRESS, 和DEVICE_END_ADDRESS, 另外还要将kernel_end_address 改成虚拟地址, config里的部分高位地址都要改改.
(MMIO表示memory mapped io. 访问这里的地址就是直接与外设交互)

## lab4线程与调度

这个lab工作量非常大.

### 总览

ucore中把初始化的执行包装成idleproc, 调用创建内核线程的函数创建init_main线程. 不过idleproc除了初始化外没有执行任何实质性的任务, 不存在也没有关系.
我们在rcore-tutorial中则直接切换到新来的线程. 而且切换后甚至原本我们使用的栈 bootstack也可以被回收.

当前的内核由于只用一个单核cpu, 只有一个内核栈, 并且不支持中断的嵌套. 而且现在中断的时候无论是内核态还是用户态都会交换sp和sscratch, 这就导致如果嵌套会导致交换两次出现问题. 需要改进为用户态切换栈, 内核态不切换栈才能为支持嵌套中断打基础.

进程和线程辨析: 线程是运行和调度的单位. 进程则包含了地址空间, 同一个进程的不同线程的地址空间是共享的.(意味着高位地址处会映射多块栈给不同的线程) 新建线程和Context都需要传入process结构体

切换则直接通过保存当前中断栈上的Context, 把下一个要执行的线程的Context放到栈顶实现.

### 代码变化梳理

1. interrupt/context.rs 完善之前的Context实现. Context结构体不变, 为Context实现了Default, 使用全零作为Default. 实现了获取/设置 栈指针/返回地址的四个简单函数. 实现了按照调用规则把参数写入Context内对应寄存器的函数, 和传入函数地址, 参数, process结构体新建Context的函数. 
2. 新建process文件夹作为mod
3. 增加全局Processor用到的Lock, 原本使用的是unsafeWrapper, 在algorithm目录内.
    1. config.rs 包含了每个线程的运行栈大小, 和共用的内核栈大小, 目前都是512K
    2. process.rs process结构体当前只有is_user标志位和memory_set内存空间. 有三个函数, 新建内核进程的new_kernel, 从elf创建进程的from_elf函数(之后的lab才会添加), 映射新的虚拟地址的alloc_page_range函数(类似mmap)
    3. kernel_stack.rs 内核栈也就是作为一个大小为KERNEL_STACK_SIZE的u8数组. 此外实现了push_context函数, 能在栈顶减去Context大小的位置强转为Context指针, 然后赋值写进去, 最后把这个指针返回. 同时暴露出全局变量作为共用的全局内核栈.
    4. thread.rs Thread结构体包含id, 栈(虚拟地址range), 所属进程(arc+读写锁包装的Process结构体), 和inner(用一个mutex包装一些可变数据结构). inner包含context和是否进入休眠的sleeping标志. 实现了Hash Eq这两个trait, Debug打印的trait. prepare函数用于准备执行该线程, 会激活页表, 清空并返回Context, park函数会暂停线程, 保存传入的Context. 新建线程的new函数需要传入Process, 要执行的entry_point, 和参数, 该函数会新分配一段空间(alloc_page_range)作为栈, 并构建新的Context, 最后打包新建thread并返回
    5. processor.rs 包装调度器算法, 包装进程状态转移的操作.
3. 增加新的调度算法: 使用hrrn高响应比优先的调度算法, 放到process文件夹内. hrrnThread结构体对线程再次包装, 增加birth_time和service_count两个字段. 调度结构体HrrnScheduler则包含linkedList保存的hrrnThread和currenttime的二元组.
2. 修改interrupt.asm支持切换线程, 加入交换sscratch的代码, 修改保存sp为保存sscratch. 恢复时保存弹出Context之后的栈到sscratch. 把a1放到sp使得`__restore`有返回值和参数这两种新的调用方法, 从而执行不同的线程.
3. 修改interrupt_handler (init函数里面的增加各种中断使能的操作先不做, 后面的lab需要键盘输入的时候再加上), 修改时钟中断处, tick之后调用保存当前进程和准备下一个进程的函数, timer模块不变.
6. 在interrupt/mod.rs中增加一个wait_for_interrupt函数. 给processor.rs中函数调用
3. 修改timer模块init, 删除`sstatus::set_sie();` 这样main函数就不开中断, 执行内核线程的时候再接受中断
3. 修改main.rs启动线程 main函数首先对各种东西进行初始化, 然后对线程的实现进行测试.

### 实现细节

#### interrupt/context.rs
这里新建的时候Context中返回值寄存器设置为-1, 如果执行的函数返回了, 就会报错. 之后新版的代码似乎修改了这里, 能正常返回.

#### process/process.rs

process结构体也算是MemorySet的封装了, 新建的时候会新建MemorySet, 函数也是调用MemorySet的接口.

alloc_page_range函数类似于mmap吧, 基于memory_set提供的接口操作, 传入一个size, 返回分配好的地址范围. 首先把size向上取整到页倍数, 再用while循环从0x100_0000开始不断步进查找可用内存空间(memory_set的overlap_with), 可用则调用memory_set.add_segment增加映射.

#### kernel_stack.rs

因为线程都有自己的栈映射在低地址区了, 函数调用关系的维护不靠内核栈, 内核栈只处理中断, 而且中断不会嵌套.
如果中断不嵌套, 那么Context总是在公用内核栈最顶上. 因此切换内核线程前可直接将Context放到栈顶上假装是正常的中断返回.

#### thread.rs
prepare函数, 1激活了新线程的页表, 2把Context放到了公共内核栈的栈顶.
切换线程的时候都是这样假装是中断返回. 此时大概调用了park函数保存好了中断进入时那个线程的Context.

重点关注新建线程的函数.
1. 新建线程的时候用的栈, 是新映射分配的!!
2. 新建的Context, 在切换的过程中还不会打开中断, 直到sret的时候中断才会打开.
3. 是否是用户线程取决于Process的is_user.
4. 返回的时候, is_sleeping是false. 这意味着一创建就开始执行? 不, 这只说明不是放到sleeping的队列而是放到scheduler的队列里

#### processor.rs
processor 主要是包装一下进程的状态转换操作, 调度靠每次prepare_next_thread中询问调度算法的操作, 主要靠timer_interrupt的时候的park+prepare组合拳. 当线程出现问题调用fault函数的时候, 就会调用kill/prepare组合拳

在这样单核的环境, 确实可以说调度器是局部于处理器的.

线程的组织上, 使用了hashbrown这个crate保存sleeping的thread, 需要调度的线程让调度算法去用自己的数据结构保存.

add thread会检查是否当前线程为None, 是则放入. 主要还是加入scheduler. 只有刚启动的时候, 和kill线程的时候会take当前的Option-thread 为none

确实, \_\_restore作为函数调用只出现在processor的run里面, run函数只出现在rust_main里面. 只有刚开始的时候会刻意去调用\_\_restore 毕竟,初始化的时候并不是调用的interrupt_handler, 不会返回到__restore. 而且使用的是boot_stack


执行的函数组合可以为 parl/kill \[sleep\] prepare_next_thread

#### interrupt.rs
外部中断如果是键盘输入, 就会把字符push到STDIN里(fs/stdin.rs)TODO

#### unsafeWrapper
unsafe包装后能增加多少灵活性? 这里的unsafe一方面实现了&self 转mut, 使得可以同时持有多个可变借用. 并且实现了(标记了)sync Trait, 使得可以多线程共享.

static unsafe wrapper 还增加了Phantom data成员, 表明该结构体确实拥有T类型的值的所有权, 让wrapper被drop的时候也能drop T.


#### hrrn_scheduler.rs
alloc::collections::LinkedList组织内部线程
对线程再增加了一层包装HrrnThread, 增加相关数据结构.
真正关键的只是一个.max_by()

#### interrupt.asm

原来是直接把sp保存不切换栈, 现在是先交换sscratch再保存, 而且保存的时候是保存的原来的sp, 所以恢复的时候直接正常恢复就好. 只要把当前的栈放好到sscratch就行.

无论如何, 发生了中断就交换栈为sscratch. 如果是单线程不嵌套中断的话, 一定是公共栈顶上吧


riscv调用约定中, a0, a1既是第一个参数, 又是返回地址. 这设计强!!
```
__restore:
    mv      sp, a0  # 加入这一行
```
这样一方面可以调用__restore(context), 一方面也可以让interrupt_handler返回context指针. 也就是中断返回的时候, 如果不切换进程, 就返回当前的进程的context, 否则返回切换到的进程的context.
其次, 把第一个参数作为sp, 而sp不仅是当前Context的位置, 还是之后保存到sscratch的位置!!! 因此这个参数/返回值一定要放在作为栈的内存顶上

#### interrupt/handler.rs
这里打开了一些神奇的中断. `sie::set_sext();`这个应该只是使能sie寄存器的某个中断, 中断的总开关还是没有打开的. 

handle_interrupt函数中, 每个单独的处理函数确实应该返回应该Result类型, 是错误则调用fault.

fault函数现在出现异常的时候会杀死当前的线程了, 传入的参数也变了
最重要的当然还是timer的时候调度一下

#### lock.rs
为Mutex增加关中断的功能得到Lock类型. 则当获取其中内容的时候, 既关+保存了中断, 又独占了资源.

具体实现上, 上锁是在get函数中, 释放是在Drop的trait中.

同时实现了deref和deref mut, 可作为被包裹对象的智能指针使用.

另外实现了一个不上锁不关中断, 直接获得内部对象的unsafe_get函数, 用于PROCESSOR::run()的时候因为不会返回, 导致不会调用对应的析构函数

### 线程的结束

目前, 内核线程的结束是靠设置自己线程的isDead变量, 然后触发中断的时候检查时结束的. 也就是说设置了这个变量后即使不使用ebreak, 用其他方式触发中断也会被结束.

### 线程/进程的保存与组织

线程的组织上, 使用了hashbrown这个crate保存sleeping的thread, 需要调度的线程让调度算法去用自己的数据结构保存.

Arc RwLock包着进程, 创建新线程的时候会把Arc RwLock-进程的所有权要过来, 用clone可以多处持有. 似乎没有单独组织进程的地方, 父进程子进程之类的关系链接也似乎没有

## lab5 设备驱动与文件系统

### 综述
1. 提供对ELF文件用户程序的解析
2. 提供对磁盘映像的支持
3. 实现系统调用支持
4. 支持文件描述符

### 相关修改

0. entry.asm 增加映射低位页表
1. main函数增加参数, 调用drivers的初始化函数
2. 新建drivers文件夹, 在mod.rs中增加init函数, 调用设备树子模块的init函数
3. drivers/device_tree.rs 增加对device_tree这个库的依赖. init函数调用dtb遍历库, walk函数负责遍历生成的树.
4. 修改makefile, 增加qemu的启动参数 增加TEST_IMG变量指向之后的磁盘镜像
5. 增加drivers/bus文件夹并增加子模块virtio_mmio. 

#### entry.asm 
这里没想到增加了新的页表. 花了我差不多一个小时的时间去debug
当parse dtb, 得到磁盘的Header的时候, 这个header的位置在0x1000_8000, 位于低地址, 而我们此时还处于初始化的状态, 映射还是巨页映射, 只映射了0x8000 0000开头的1GB, (1GB是0x4000 0000). 
更加恐怖的是, 此时的sscratch都没有值, 中断都无法正常进行. 
当前的设计是中断一定切换栈, 每次中断的时候都是先交换sscratch和sp, 然后保存真正的原来的sp到栈上. 离开的时候先把sp弹出后的位置放到sscratch. 而第一次放入sscratch就是运行线程时假装中断返回的时候.
因此, 我在debug的时候, 第一次页访问异常, 进入中断的时候, 从sscratch中取来了(似乎是-1)未知的值, 作为内核栈放到sp, 然后在保存第一个寄存器的时候发生了第二次页访问异常. 这时又把sscratch和sp交换, 得到了正常的sp值, 反而正常处理了, 但是报错的地址好像是0xffff_ffff_ffff_ffec.
我在main函数加入了把kernel stack放到sscratch的汇编才正常得到了中断发生错误的地址.

#### drivers模块

driver模块目前主要负责文件系统的driver. 对外暴露的接口是driver模块中的driver trait, 和DRIVERS这个组织各种驱动的数据结构.

模块的初始化函数在mod.rs中, 参数传入dtb地址, 负责调用解析设备树的device_tree::init, device_tree的walk函数则一边遍历, 一边判断是不是想要的设备, 这里单指对应磁盘的块设备, 找到则把这个节点传给对应的总线协议驱动程序, 这里是bus/virtio_mmio. 本次实验中找到块设备的节点后, 把节点里reg字段拿出, 转换为virtio_drivers(库)::VirtIOHeader类型, 就传给驱动程序的包装virtio_blk, virtio_blk::VirtIOBlkDriver内部包装的是mutex包装的virtio_drivers(库)::VirtIOBlk, 对外实现了driver的trait.

之后在fs中才会用到BlockDevice, 它包装driver, 实现rcore_fs的BlockDevice trait从而传入给SimpleFileSystem::open函数

层次关系:
virtio_drivers(库)::VirtIOBlk --包装--> VirtIOBlkDriver(实现Driver) --包装--> BlockDevice(实现rcore_fs的BlockDevice)

#### driver.rs
这个模块包含了驱动接口driver trait, 描述驱动类型的DeviceType, 和lazy_static的DRIVERS.

Driver 这个trait, 使用时根据device_type返回的DeviceType, 来调用对应的方法, 现在这个trait中只有块设备相关的方法, 提供了用unimplemented宏表明没有实现这个方法的默认实现. 实现新的设备驱动的时候, 只需要加入新的方法即可.

DRIVERS保存所有驱动的数据结构, 方便获取驱动. 驱动是dyn Driver类型, 首先用Arc实现共享, 再通过Vec保存, 再加上读写锁保证多线程安全.

#### device_tree.rs

device tree blob 确实是个标准. 其中头部的字段其实很多, 包括了dtb的版本等等. 我们现在只读取了前两个字段. 第二个字段size确实是包含整个dtb的大小的, 包括头部.
https://www.devicetree.org/specifications/

这里init函数首先校验头部, 得到size, 再把整个dtb作为u8数组传入DeviceTree的crate中, 并且让遍历的walk函数递归遍历.
而walk函数则只是搜索compatiable字段为virtio,mmio的节点, 把节点传入virtio_probe进行初始化

#### bus/virtio_mmio.rs
将传入的dtb节点的reg字段转为VirtIOHeader传入驱动程序进行初始化.

这里会遇到不少verify不对的设备, 因此如果verify函数调用失败或者没有reg字段就直接返回. 这里verify的时候就会访问1000_xxxx开头的低地址.

此外, 暴露了virtio_dma_dealloc, virtio_phys_to_virt, virtio_virt_to_phys这三个extern "C"且no_mangle的函数. 而且没有在我们代码中其他地方被调用. 这说明是库函数在C语言中或者汇编中调用了这两个函数. 根据名字看可能是virtio库.

#### block模块和 block/virtio_blk.rs
block模块的mod.rs里提供了对接驱动与文件系统的VirtIOBlkDriver包装. 而模块内部则是保存的块设备的Driver实现.

BlockDevice的实现主要是将Driver的返回bool值的read_block/write_block 函数转换成返回Result<()>的read_at/write_at函数, 另外实现假的sync函数

VirtIOBlkDriver的实现就是调用内部的Driver的read_block/write_block函数, 把返回的Result再用is_ok转成bool.

VirtIOBlkDriver则需要实现read_block/write_block的Driver接口, 另外给解析Node的virtio_mmio.rs:virtio_probe函数一个创建设备的add_driver函数. add_driver函数把header传给VirtIOBlk::new得到包装的内部驱动, 再把驱动包装上刚刚实现的VirtIOBlkDriver加入DRIVERS列表.

### fs模块

模块的mod.rs 提供了lazy_static的全局变量ROOT_INODE, 初始化的时候获取第一个Block类型的driver, 用BlockDevice包装后传入SimpleFileSystem::open()函数, 返回值赋给ROOT_INODE. 也许是SimpleFileSystem实现了对INode的deref, 在ROOT_INODE上可以调用到inode_ext拓展INode实现的方法

还有init函数, 负责作为测试, main函数初始化的时候使用ls方法测试我们文件系统的功能.

#### inode_ext.rs
impl INodeExt for dyn INode 通过这种方法为已有的Inode类型增加功能. 额外实现了 ls这个直接打印而不返回值的函数, 和readall函数, 把所有内容读到`Vec<u8>`并返回.

### elf相关代码

ELF文件也可以看作是一个地址空间. 因为它定义了各个段的映射关系.
MemorySet中增加根据ELF文件创建的from_elf函数, 它遍历elf文件的每个段, 根据大小和权限映射每个段.

首先在program_header中对每个为load类型的段, 读取开始地址和大小和数据和权限之后进行映射

process中也加入from_elf函数, 主要是调用MemorySet中的from_elf函数.

### sfs文件系统中的指向关系

文件夹inode -> 文件entry -> 文件inode -> 文件数据

## lab6 用户程序, 系统调用, 文件描述符

### 步骤
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

### 用户进程

类似于自己做一些rust标准库的事情. 
首先是实现了ecall的包装,从而实现了sys_read, sys_write, sys_exit. 利用sys_write实现了print, println宏. 实现了对用户程序的输入输出的支持.

hello_world主要使用了输出.
nodebook可以把输入的字符回显.

#### fs/stdout.rs stdin.rs

让标准输入和输出实现和文件一样的接口(INode)进行读写.

stdout没想到就是一个空结构体
```rust
pub struct Stdout;
```
然后直接实现INode的方法, read和poll都返回不支持的错误, write不允许offset为非0.

标准输入stdin同理, 只允许offset为0, buf中没有值则等待一次条件变量, 否则进入读过程, 要么是stdin的buffer空了, 要么是buf不够长, 返回.

#### 系统调用的实现

syscall_handler函数根据传入的系统调用号调用各个子函数, 重要的是子函数的返回值还代表了对当前进程的处理方式.

write: 根据fd在进程的descriptor内获取inode, 调用inode的write_at, 直接返回Proceed和返回值.

read: 调用inode的read_at, 然后根据返回值包装一下. 和write不同的地方在于, 如果返回值为0则park当前线程(阻塞), 此时已经在read_at内部等待了condvar, 调用等待时会把当前线程放入等待队列并sleep_current_thread. 之后syscall_handler在处理返回值的时候发现是Park类型会再切换线程.
直到之后external interrupt键盘输入->push到stdin中->条件变量notify->进程恢复调度.

#### condvar
这里的条件变量利用的是线程的休眠, 等待条件变量时进入条件变量内部的队列, 线程休眠. 当notify时则唤醒进程.
只在fs/stdin.rs中被实例化并使用.

如果有多个线程同时等待标准输入? 因为只会notify_one, 因此每次只有一个进程会醒来读取标准输入进行服务.

## 实验题

### lab4 上

实现在
https://github.com/am009/rCore-Tutorial/tree/lab-4-challenge
这个repo里面新建立的challenge结尾的分支上

#### 按Ctrl-C杀死当前线程
在处理外部中断读取键盘字符时, 加入打印的函数, 发现当按下Ctrl-C会产生ascii编码为3的控制字符, 在此处判断如果字符等于3的话就杀死当前线程(和fault函数一样).

#### fork的实现

写得比较晚还是有好处的. 现在的实现直接既复制进程又复制线程.
目前还没有写时复制机制, 打算把低位地址全部复制了去.

1. 为thread增加clone_thread方法. Context实现了Clone, 就直接调用. 关键在于process的复制
2. 为process实现clone的trait, 关键在于memory_set. descriptor则遍历调用clone复制(STDIN/STDOUT都是arc的)
3. 为memory_set实现clone的trait, 每个用户进程都是基于内核的映射的, 先建立内核映射, 再依次复制其他映射(segments实现了Eq), 可以发现栈和其他映射都是Framed类型的.
4. 关键在于内存地址的复制, 可以直接调用memory_set的add_segment, 问题是初始数据. 

如果页表就是当前页表, 可以直接根据虚拟地址复制.
如果页表不一定是当前页表, 那就需要根据页表找到对应的物理地址, 并且(利用线性偏移)访问物理地址获取数据. 
这查找的函数, 有类似的lookup函数, 但是也是需要当前页表, 是satp寄存器里的

#### 实现支持非当前页表的进程拷贝

首先对mappings增加lookup_self函数, 在lookup的基础上修改, 只不过使用的不少satp寄存器, 而是自己保存的页目录表的页号来查找.

memory_set增加clone_segment函数, 类似add_segmen函数, 但是调用的是mapping的map_clone函数(而不是原有的map函数)

最后mappings增加map_clone函数, 在map的基础上修改, 不过不是从传入的data获得数据, 而是从传入的另外一个mapping调用lookup_self获取物理地址, 用线性偏移访问得到数据.

#### 测试
将hello改为这个程序, 能够有效测试实现对不对.
```rust
pub fn main() -> usize {
    println!("Hello world from user mode program!");
    for i in 1..0x7fffffusize {
        if i % 0x100000 == 0 {
            println!("hello");
        }
    }
    0
}
```

按久了还会报内存不足的错.
```
src/memory/mapping/memory_set.rs:29: 'called `Result::unwrap()` on an `Err` value: "no available frame to allocate"'
```

### lab6

#### sys_tid 系统调用
写系统调用要写两边, 一边是操作系统端, 另外需要给用户端包装为适合使用的函数.
这个太简单了.

#### sys_fork 系统调用
当调用我实现的fork函数, 需要先park当前线程(使Context不为空,这样就能复制Context了), 再fork, 结束时调用prepare_next_thread.
调用sys_fork时处于syscall的trap, 因此park后修改fork前的线程和fork后的线程的Context中对应寄存器就可以修改fork的返回值了.

`相比于实验四，你可能需要额外注意文件描述符的复制。` 这我怎么没有感觉到? 不过之前就是遍历并且调用clone的, 不知道INode有没有实现clone, 打开文件再fork会怎么样.

本来打算改下系统调用号, 但是看到只有openat没有open, 也没有fork就算了.
http://git.musl-libc.org/cgit/musl/tree/arch/riscv64/bits/syscall.h.in

#### sys_open

遇到的第一个难点就是传递字符串. 我第一个想法当然是使用C风格的00结尾的字符串, 这样感觉兼容性更好, 网上搜了一下发现std::ffi中有方便的转换函数. 然而仔细搜索发现, no_std时不能使用, core::ffi还不完善, 基本上是空的, libc这个rust的crate是分架构的, riscv-imac那个里面也是空的.
https://github.com/rust-lang/rust/issues/46736
如果直接传递字符串指针行吗? 将参数改为 *mut str类型发现不能从usize转换, 因为字符串指针是胖指针.
我突然想到string有从u8的buffer构建字符串的函数, 那我可以传u8指针和size, 合成出buffer,再转为字符串. 
现在暂时使用utf-8编码, 不知道sfs是不是支持, 只要不使用特殊字符就和ascii是兼容的. 
之后考虑兼容C风格的00结尾的字符串, 拿着指针向后一个个找直到00结尾的地方, 这样就能找到size, 不过客户端编程不太方便, 先不实现. 

考虑inode在fork时的复制问题. find得到的INode都是Arc包装的. 也就是指向的是同一个INode. INode是一个trait, 而不是结构体, 这导致不一定能够复制内部的结构体.
```rust
/// INode for SFS
pub struct INodeImpl {
    /// INode number
    id: INodeId,
    /// On-disk INode
    disk_inode: RwLock<Dirty<DiskINode>>,
    /// Reference to SFS, used by almost all operations
    fs: Arc<SimpleFileSystem>,
    /// Char/block device id (major, minor)
    /// e.g. crw-rw-rw- 1 root wheel 3, 2 May 13 16:40 /dev/null
    device_inode_id: usize,
}
```
但是这个sfs的实现似乎没有什么seek之类的成员标识当前位置, 读取和写入似乎不会影响状态, 那似乎就可以引用同一个对象了, 应该没什么问题吧. 不过似乎读写有缓冲...

最后算是成功实现了吧, 这里INode的读写都是带offset的, 似乎没有什么当前的指针.

#### sys_pipe
类似STDIN去实现INode, 有一个缓冲区, 写入满或者读完后阻塞.

首先遇到的难点是系统调用怎么返回两个值? 一时没有什么好的解决办法, 只好在用户态增加一个syscall2函数, 内核增加一个syscall的返回类型Proceed2.

模仿stdin的处理方式. 如果写而缓冲区满, 读而缓冲区空的时候, 就返回0并且让线程在条件变量的时候等待. 在另外一边响应的时候notify_one.

最终的设计是, PipeInner里包含缓冲区(`VecDeque<u8>`)和两个条件变量, 使用Mutex和Arc包装. PipeReader和PipeWrite分别持有一份Arc的clone拷贝.
两个条件变量, 一个用来等待写, 一个用来等待读. 每当读/写了1次就notify对应的条件变量. 有个问题就是一次写很多而很多进程读, 每个进程读得很少的时候, 当notify_one的时候如果一个进程没有读完就走了, 就不会继续唤醒其他线程了. 改成notify_all可能好一些? 不过改起来也简单. 再说吧. TODO
本来是打算用读写锁加缓冲区的, 后来发现可以直接和stdin一样用一个队列(VecDeque), 这样无论读还是写都需要修改缓冲区了, 就换成了Mutex. 

又开始了漫长的debug. 第一次是给sys_write增加判断等于0的条件的时候忘了把上面的大于等于0的判断改成大于. 第二次是make的时候在改user, 发现怎么改都无法进入下面的语句的时候才发现根本就没有编译user那边...

最后效果确实还行. 一边读, 另外一边写, 可以相互切换. 写的那边退出了, 读的那边还在sleep. 如果用Ctrl+C杀死的话, 由于sleep的线程不在schedule里, 导致无法找到, assertion failed, 最终panic. 只要读的那边让它少读一点就可以正常退出了. 用户程序那边是用得相同的缓冲区, 导致有时没有读满也打印了之前的字符.

下面是输出, 时钟中断的随机发生有时候还打断了println的输出? 可能是println分多次输出的过程中被打断的. 
```
mod memory initialized
mod interrupt initialized
mod driver initialized
.
..
tmp.txt
hello_world
notebook
mod fs initialized
Hello world from user thread 1!
file desc: 2
read ret: 3
get buf 789
hello from parent!
process sleeped: 0xffffffff80282860
hello from child!
process waked: 0xffffffff80282860
write pipe success: read pipe success: 11
content: hello_world
process sleeped: 0xffffffff80282860
11
write once.
process waked: 0xffffffff80282860
read 8 bytes : abcdefgh.
process sleeped: 0xffffffff80282860
write once.
process waked: 0xffffffff80282860
write once.
write once.
write once.
write once.
process sleeped: 0xffffffff80282838
process waked: 0xffffffff80282838
pipe full on write 5.
write once.
write once.
process sleeped: 0xffffffff80282838
read 8 bytes : abcdefgh.
process waked: 0xffffffff80282838
read 8 bytes : abcdefgh.
read 8 bytes : abcdefgh.
read 8 bytes : abcdefgh.
read 8 bytes : abcdefgh.
process sleeped: 0xffffffff80282860
pipe full on write 7.
write once.
process waked: 0xffffffff80282860
write once.
thread 1 exit with code 0
read 8 bytes : abcdefgh.
thread 2 exit with code 0
src/process/processor.rs:87: 'all threads terminated, shutting down'
make[1]: Leaving directory '/home/wjk/os/rCore-Tutorial/os'
```

### lab4 下

stride scheduling 这不就是我ucore止步的地方... 由于溢出, 如何判断开始和结束那里有点复杂. 还是ucore的文档说的详细, 还给出了很多资料. 

pass指的是当前的地点, stride指的是一次走的大小. 当优先级大于等于1的时候, 进程的stride就最大是 BIG_STRIDE, BIG_STRIDE感觉要小于整数最大值的一半. 
由于各个进程的pass都聚集在相关的BIG_STRIDE范围内. 最靠前的进程, 向前BIG_STRIDE的范围内都没有其他进程, 最落后的进程, 加上BIG_STRIDE的范围内有所有进程. 怎么在这个(模2的n次方的有限域)内找到最落后的进程?? 只要拿两个进程相减, 差也通过模运算放到有限域内, 根据是否最高位来判断是大于0还是小于0.

假设在mod16的域上, big_stride是4. 有进程的pass分别是15, 16 ,0 ,1. 用1 - 15 (mod 16)得到2, 因此1在15 前面

没想到实现起来不算难, 还算简单, 在alloc::collections里找到了BinaryHeap作为优先级队列. 现在就差如何设置global_pass了. global_pass的设置没怎么看懂...暂且设置为当前最小的那个吧...

仔细看了看论文, 发现无论是动态修改优先级的时候动态修改pass, 还是维护一个global_pass, 都比想象中复杂一些... 实现得还不够好. 明天再继续吧


```
hello from kernel thread 2
hello from kernel thread 3
hello from kernel thread 1
hello 3
hello 2
hello 3
hello 3
hello 2
hello 1
hello 3
hello 2
hello 3
hello 3
hello 2
hello 1
hello 3
hello 2
thread 3 exit
hello 1
hello 2
hello 2
hello 1
thread 2 exit
hello 1
hello 1
hello 1
thread 1 exit
```