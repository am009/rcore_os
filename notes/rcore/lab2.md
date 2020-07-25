# lab2内存管理

包括临时堆内存管理, 物理内存管理.

## 临时堆内存管理
### 分配接口的使用

ucore中是先实现按页的物理内存管理, 再实现的任意大小的管理的. 而这里完全不一样, 先是bss段留了8M空间作为堆, 给操作系统动态内存分配用, 再是实现的按页的物理内存管理.

这里暂时使用临时堆内存管理这个新词, 表示为了使用rust提供的一些需要使用堆内存的数据结构而在bss段上划分出一块空间作为堆. 

我们分配算法和rust的对接主要在于Trait GlobalAlloc, 实例化之后用 `#[global_allocator]`标记就可以使用动态内存分配了(可以使用一些需要动态内存分配的内置数据结构, 如Box, Vec等). 接口也是和C语言中malloc/free类似的接口: alloc和dealloc.

### 分配的实现
而rcore-tutorial这里我们直接使用现有的buddy system内存分配算法. 而且本次实现的是一次分配

在rcore中开辟8M大小的bss段, 作为用来分配的空间.

### bss+buddy system内存分配

直接分配u8 static数组,数组名字指向的就是对应的空间.
添加buddy system这个包. spin和lazy_static也顺便加上.
创建memory文件夹作为新的mod, 创建init函数被main调用. 创建一个HEAP全局变量作为分配器, 并在init函数里面把那个数组的内存初始化给它. 
想把数组的名字作为指针, 只需要调用.as_str()然后as转换为usize就可以了.
这样, main函数调用完heap的init之后就可以分配堆空间了.


## 使用自己的堆分配算法

答案中的heap2使用自己的algorithm crate的bitmap_vector_allocator提供分配算法支持, 这里自己实现简单的分配算法也很简单, 代码量挺少的. 

堆分配算法对外的接口一个是`#[global_allocator]`标注, 一个是init函数.
使用的是bitmap标记空闲, 以字节为单位, 查找时按照对齐要求的倍数顺序查找(作为内存的开头), 直到遇到了空闲处.
只标记4096字节, 最多只能管理4K的内存. 这里的实现也是对给定内存范围的对应内存的index(偏移)做分配, 每次分配得到的只是一个偏移, 需要去找到对应的内存地址.

使用自己实现的堆临时分配, 可以把旧的heap.rs重命名为heap_old.rs 新建heap.rs对算法封装. 不像buddy_system实现好了GlobalAlloc, 为了global_allocator要实现alloc::alloc::GlobalAlloc这个trait. 两个函数, 传入的是core::alloc::Layout, 并且需要处理指针类型 *mut u8. 直接整个实现都是unsafe的. Heap全局变量也不再是简单的直接是一个实例, 而是在VectorAllocatorImpl的基础上包了一层Option, 再包UnsafeCell. UnsafeCell需要get再as_mut, Option就unwarp, 就可以得到内部的VectorAllocatorImpl调用alloc/dealloc函数.

Option可以提供默认值. 初始化之前, Option里是None, 初始化函数使用replace函数替换成实例之后才能分配, 否则会在unwrap的时候panic

完成之后, 将堆测试的两个循环数量从10000减少到100才能正常通过测试. 看来还是不太好用, 切换回之前的buddy system算法吧.


## 按页的物理内存管理

物理内存管理不像临时堆内存管理只是为了让我们操作系统自己用, 它也是虚拟内存管理的基础. 否则只要直接把LockedHeap的init函数中传入的内存改成我们可用的所有内存范围, 就能用上这些内存了.

### 封装地址类型与frame tracker
内存地址空间确实是非常常用的东西. 无论是分页内存管理还是mmio, 之后肯定会大量用到内存地址的. 因此这里封装出了地址类.
1. 封装地址类和页号类
1. 抽象实现From trait在地址和页号间相互转换
1. 抽象实现地址转页号的向上/下取整
1. 抽象实现和usize的加减输出操作.

还要在memory中新建mod range. 提供对内存地址range的支持. 它在基础的core::ops::range基础上增加相互转换, 和len函数, 和迭代, 重叠检测, 虚拟物理相互转换, 取下标, 包含检测的功能.

frame tracker 作为物理页面的智能指针, 继承PhysicalPageNumber, 实现Drop.
frame模块除了frame_tracker的实现, 同时承载分配相关的实现: allocator.rs 对分配算法进行包装, 对接frame tracker和分配. 分配算法实现Allocator trait(一次分配回收一个元素的index), 包装后提供初始化创建(起始物理页号和页数量), 分配一个页(frame tracker), frame tracker析构的时候自动调用回收.
由于是简单的一次分配一个元素, 而且可以是离散的, 简单地使用一个栈进行分配. 创建StackedAllocator, 在allocator.rs中作为AllocImpl, 就会使用这个算法了.

frame tracker创建的时候不会自动申请页面, 因此想要获得frame tracker需要通过allocator分配, 不能自己构造

### 分配哪些内存
可以直接根据qemu内存映射, riscv从0x8000_0000开始到0x8800_0000这128M初始内存, 直接硬编码拿来. 设置全局变量KERNEL_END_ADDRESS, MEMORY_START_ADDRESS, MEMORY_END_ADDRESS, KERNEL_HEAP_SIZE同理. 最终

在ucore中, 一般以函数指针结构体作为接口, 让不同的分配算法提供相同的接口. 而且花大量精力, 使用Page结构体, 链表组织空闲页面.(物理内存管理)

### FrameAllocator

实现单页单页的物理内存的分配和回收. 内部使用的算法是StackedAllocator, 非常简单, 一个Vec, 新建的时候把一段物理内存范围输入, 每次分配取栈顶的一页, 每次回收页就压入栈中.
接着在StackedAllocator基础上包装出FrameAllocator, 把对下标的分配转化成真正的内存地址, 并且返回时返回Frame Tracker.

物理内存的分配器目前只实现的单页单页的分配和回收, 这一点我其实觉得挺合理的, 因为现在离散式虚拟内存技术已经非常成熟, 我们操作系统用到的现在全都是虚拟地址, 需要"连续的内存"时也一般是需要连续的虚拟地址空间, 因此物理地址的分配完全可以是这样一页一页离散的. 
但是之后的lab可以看到, DMA确实出现了需要连续的物理内存空间这种需求, 看来这里的实现确实值得改进.

## 问题
context.rs中给context结构体实现了Default trait, 使用全零. 这里不实现会怎么样

为什么KERNEL_END_ADDRESS是lazy_static?
因为不用会报错"pointer-to-integer cast" needs an rfc before being allowed inside constants

下面这个impl是什么意思?
```rust
pub fn iter(&self) -> impl Iterator<Item = T> {
    (self.start.into()..self.end.into()).map(T::from)
}
```

下面这句话的下划线是什么意思??
```rust
let offset = ptr as usize - &HEAP_SPACE as *const _ as usize;
```


## 用到的其他知识点
rustdoc的注释写法里`//!`是什么意思?
表示对整个模块的doc

print trait的实现
抽象实现的时候的宏的用法, 宏参数的几个类型

https://doc.rust-lang.org/reference/visibility-and-privacy.html
pub(super) 对父模块公开

我以为只需要划好一块内存地址空间给rust, rust就能自动帮分配(和之前print宏一样), 仔细想想确实不好, 内存分配确实比较底层, 而且要考虑使用的算法, 不同的算法有不同的性能.
没想到Rc, arc这两个数据结构也和box一样在堆上分配内存


思考题有些神奇, 不把申请到的页面的所有权传到外面, 就会死锁.
