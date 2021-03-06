# lab4 线程与调度

这个lab工作量非常大.

## 总览

ucore中把初始化的执行包装成idleproc, 调用创建内核线程的函数创建init_main线程. 不过idleproc除了初始化外没有执行任何实质性的任务, 不存在也没有关系.
我们在rcore-tutorial中则直接切换到新来的线程. 而且切换后甚至原本我们使用的栈 bootstack也可以被回收.

当前的内核由于只用一个单核cpu, 只有一个内核栈, 并且不支持中断的嵌套. 而且现在中断的时候无论是内核态还是用户态都会交换sp和sscratch, 这就导致如果嵌套会导致交换两次出现问题. 需要改进为用户态切换栈, 内核态不切换栈才能为支持嵌套中断打基础.

进程和线程辨析: 线程是运行和调度的单位. 进程则包含了地址空间, 同一个进程的不同线程的地址空间是共享的.(意味着高位地址处会映射多块栈给不同的线程) 新建线程和Context都需要传入process结构体

切换则直接通过保存当前中断栈上的Context, 把下一个要执行的线程的Context放到栈顶实现.

## 代码变化梳理

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

## 实现细节

### interrupt/context.rs
这里新建的时候Context中返回值寄存器设置为-1, 如果执行的函数返回了, 就会报错. 之后新版的代码似乎修改了这里, 能正常返回.

### process/process.rs

process结构体也算是MemorySet的封装了, 新建的时候会新建MemorySet, 函数也是调用MemorySet的接口.

alloc_page_range函数类似于mmap吧, 基于memory_set提供的接口操作, 传入一个size, 返回分配好的地址范围. 首先把size向上取整到页倍数, 再用while循环从0x100_0000开始不断步进查找可用内存空间(memory_set的overlap_with), 可用则调用memory_set.add_segment增加映射.

### kernel_stack.rs

因为线程都有自己的栈映射在低地址区了, 函数调用关系的维护不靠内核栈, 内核栈只处理中断, 而且中断不会嵌套.
如果中断不嵌套, 那么Context总是在公用内核栈最顶上. 因此切换内核线程前可直接将Context放到栈顶上假装是正常的中断返回.

### thread.rs
prepare函数, 1激活了新线程的页表, 2把Context放到了公共内核栈的栈顶.
切换线程的时候都是这样假装是中断返回. 此时大概调用了park函数保存好了中断进入时那个线程的Context.

重点关注新建线程的函数.
1. 新建线程的时候用的栈, 是新映射分配的!!
2. 新建的Context, 在切换的过程中还不会打开中断, 直到sret的时候中断才会打开.
3. 是否是用户线程取决于Process的is_user.
4. 返回的时候, is_sleeping是false. 这意味着一创建就开始执行? 不, 这只说明不是放到sleeping的队列而是放到scheduler的队列里

## processor.rs
processor 主要是包装一下进程的状态转换操作, 调度靠每次prepare_next_thread中询问调度算法的操作, 主要靠timer_interrupt的时候的park+prepare组合拳. 当线程出现问题调用fault函数的时候, 就会调用kill/prepare组合拳

在这样单核的环境, 确实可以说调度器是局部于处理器的.

线程的组织上, 使用了hashbrown这个crate保存sleeping的thread, 需要调度的线程让调度算法去用自己的数据结构保存.

add thread会检查是否当前线程为None, 是则放入. 主要还是加入scheduler. 只有刚启动的时候, 和kill线程的时候会take当前的Option-thread 为none

确实, \_\_restore作为函数调用只出现在processor的run里面, run函数只出现在rust_main里面. 只有刚开始的时候会刻意去调用\_\_restore 毕竟,初始化的时候并不是调用的interrupt_handler, 不会返回到__restore. 而且使用的是boot_stack


执行的函数组合可以为 parl/kill \[sleep\] prepare_next_thread

## interrupt.rs
外部中断如果是键盘输入, 就会把字符push到STDIN里(fs/stdin.rs)TODO

## unsafeWrapper
unsafe包装后能增加多少灵活性? 这里的unsafe一方面实现了&self 转mut, 使得可以同时持有多个可变借用. 并且实现了(标记了)sync Trait, 使得可以多线程共享.

static unsafe wrapper 还增加了Phantom data成员, 表明该结构体确实拥有T类型的值的所有权, 让wrapper被drop的时候也能drop T.


## hrrn_scheduler.rs
alloc::collections::LinkedList组织内部线程
对线程再增加了一层包装HrrnThread, 增加相关数据结构.
真正关键的只是一个.max_by()

## interrupt.asm

原来是直接把sp保存不切换栈, 现在是先交换sscratch再保存, 而且保存的时候是保存的原来的sp, 所以恢复的时候直接正常恢复就好. 只要把当前的栈放好到sscratch就行.

无论如何, 发生了中断就交换栈为sscratch. sscratch的值可能是什么?? 如果是单线程不嵌套中断的话, 一定是公共栈顶上?? 恐怕是的


riscv调用约定中, a0, a1既是第一个参数, 又是返回地址. 这设计强!!
```
__restore:
    mv      sp, a0  # 加入这一行
```
这样一方面可以调用__restore(context), 一方面也可以让interrupt_handler返回context指针. 也就是中断返回的时候, 如果不切换进程, 就返回当前的进程的context, 否则返回切换到的进程的context.
其次, 把第一个参数作为sp, 而sp不仅是当前Context的位置, 还是之后保存到sscratch的位置!!! 因此这个参数/返回值一定要放在作为栈的内存顶上

## interrupt/handler.rs
这里打开了一些神奇的中断. `sie::set_sext();`这个应该只是使能sie寄存器的某个中断, 中断的总开关还是没有打开的. 
在某些特殊地址写入数字就能在OpenSBI中打开中断?? 为什么??

handle_interrupt函数中, 每个单独的处理函数确实应该返回应该Result类型, 是错误则调用fault.

fault函数现在出现异常的时候会杀死当前的线程了, 传入的参数也变了
最重要的当然还是timer的时候调度一下

## lock.rs
为Mutex增加关中断的功能得到Lock类型. 则当获取其中内容的时候, 既关+保存了中断, 又独占了资源.

具体实现上, 上锁是在get函数中, 释放是在Drop的trait中.

同时实现了deref和deref mut, 可作为被包裹对象的智能指针使用.

另外实现了一个不上锁不关中断, 直接获得内部对象的unsafe_get函数, 用于PROCESSOR::run()的时候因为不会返回, 导致不会调用对应的析构函数

## 线程的结束

目前, 内核线程的结束是靠设置自己线程的isDead变量, 然后触发中断的时候检查时结束的. 也就是说设置了这个变量后即使不使用ebreak, 用其他方式触发中断就会被结束.

这似乎有些不自然, x86中的ucore都可以直接使用用户的结束线程的系统调用. 但是在risc-v中, S态的ecall和U态的ecall是分开的, 在中断类型上也是对应着不同类型的中断. S态的ecall是作为sbi的调用接口的. 因此内核线程不能使用和用户线程一样的系统调用服务. (我想到了两种方式: 1. 寻找S态触发U态的ecall的方式. 2. 软件模拟中断时寄存器的变化同时跳转到中断处理函数处.)

## 线程/进程的保存与组织

线程的组织上, 使用了hashbrown这个crate保存sleeping的thread, 需要调度的线程让调度算法去用自己的数据结构保存.

Arc RwLock包着进程, 创建新线程的时候会把Arc RwLock-进程的所有权要过来, 用clone可以多处持有. 似乎没有单独组织

而父进程什么的关系链接似乎也没有看到??


## 大更新
突然出现了一次巨大的commit, 导致我不得不跟着改很多东西
Process不再用unsafe wrapper, 而是用新加入的lock
线程的结束一节相关的更新
context中默认返回地址不再是-1 而是0.
interrupt handler 里不再返回Result类型了, 退回了之前的
main函数改了很多

## 内核栈的使用

issue中曾经出现了内核栈的问题??

此外, main函数可以删去打开中断的开关, 让切换线程的时候切换的sstatus去打开中断.

## 当前方式的优缺点
当前方式: 不支持中断嵌套, 不支持多核, 共用处理中断的内核栈.
至少进程ID生成简单, 连Mutex都不用了.


## 主动的尝试:
增加tick函数, 这样当不切换的时候就不用park当前线程和prepare下一个

## 其他问题

supervisor_external这个中断处理, 一调用就console_getchar, 这个sbi调用阻塞吗??
或者只可能是键盘??

能嵌套中断吗? 为什么不能??

内核线程也共用内核栈? 内核线程发生中断的时候也切换到内核栈? 内核栈只处理中断? 

内核栈是不是不需要那么大? 
单个复杂中断嵌套调用的最大深度, 嵌套中断发生的情况.

这里的alloc_page_range函数搜索不会重叠的空间的时候, 步进是alloc_size, 是不是应该page_size好一些?
page_size 没有空洞, 但是效率低一些, 可能有更好的方法吧.

这里时钟中断每来一次就调度一次, 是不是切换得太多了

interrupt.asm中开辟 Context 的空间, 为什么是36\*8字节?? 而不是34\*8字节??

整个os对生命周期的使用情况怎么样?
park_current_thread那里会解引用了一个引用, 会夺走所有权吗????

lock.rs 中似乎是先恢复的sstatus寄存器再释放的MutexGuard吧...
https://doc.rust-lang.org/nomicon/lifetimes.html


StaticUnsafeInit 有什么用??? 目前似乎没被用到
rust的core::marker::PhantomData是什么?
https://doc.rust-lang.org/nomicon/phantom-data.html
phantomData不占真正的空间, 为了让rust更好静态分析

rust的安全性在整数溢出上做得比较好, 溢出了就直接panic了...


## x86 vs riscv 中断对比

嵌套中断不好切栈 --> 不支持嵌套中断 ---> 中断时Context总是在公共内核栈的栈底
因此利用了这一点, 无论是run还是prepare_next_thread, 都调用了prepare, 而它调用了公共内核栈的push_context

这恐怕是不能嵌套中断的一个很重要的原因??

riscv能否通过软件达到和x86相同的中断效果?? ????
x86时, 好像会从tss自动切换栈, push部分东西, riscv不会. 但riscv可以做到相同的效果, 刚发生中断的时候, 只不过是pc变了, 其他一切相关的东西都保存在当前寄存器内. sscratch就类似tss的栈指针!!!
关键在于判断中断前的特权级!! x86用硬件做了, riscv则没有. 而且还不能破坏任何寄存器, 这样就限制了对特权级的对比. 除非有地方能够保存几个寄存器, 这样就能空出几个寄存器, 取出之前的特权级进行对比, 看要不要切换栈. 
可以为每个硬件线程留一个类似tss的空间, 中断的时候暂时用这块地方当缓冲区, 放入一些寄存器的值, 就可以检查之前的特权级了.

考虑先切换到内核栈, 再判断之前的特权级是不是操作系统, 是则切回去保存Context

Vector模式的中断这个问题就迎刃而解?? 因为supervisor模式的中断被单独拿出来了?

## 如果要增加多核支持/中断嵌套支持该怎么做??

中断嵌套支持:
1. 使用vector模式的中断处理, 使得监管态不切换栈,而用户态切换
2. 进程调度的时候不能在公共内核栈上push_context, 需要修改当前的中断时保存的context

那内核线程是不是一定要用自己的栈?

多核支持??似乎只要每个硬件线程都是不支持中断嵌套的, 就不需要改上面的??

