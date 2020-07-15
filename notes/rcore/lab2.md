# lab2内存管理

似乎包括物理内存和虚拟内存

## 内存分配实现
### 分配接口的使用
我以为只需要划好一块内存地址空间给rust, rust就能自动帮分配(和之前print宏一样), 仔细想想确实不好, 内存分配确实比较底层, 而且要考虑使用的算法.
和rust的对接主要在于Trait GlobalAlloc, 实例化，并用 \#\[global_allocator\]标记就可以语法层面使用动态内存分配了. 接口也是和malloc/free类似的接口: alloc和dealloc.

没想到Rc, arc这两个数据结构也和box一样在堆上分配内存

### 分配的实现
在ucore中, 一般以函数指针结构体作为接口, 让不同的分配算法提供相同的接口. 花大量精力, 使用Page结构体, 链表组织空闲页面.
而这里我们直接使用现有的buddy system内存分配算法.

### 分配哪些内存
可以直接根据qemu内存映射, riscv从0x8000_0000开始到0x8800_0000这128M初始内存, 直接硬编码拿来
也可以在rcore中开辟bss段, 作为用来分配的空间.

## bss+buddy system内存分配

ucore中是先实现按页的物理内存管理, 再实现的任意大小的管理的. 而这里完全不一样, 先是bss段留了8M空间作为堆, 给操作系统动态内存分配用, 再是实现的按页的物理内存管理.

直接分配u8 static数组,数组名字指向的就是对应的空间.
添加buddy system这个包. spin和lazy_static也顺便加上.
创建memory文件夹作为新的mod, 创建init函数被main调用. 创建一个HEAP全局变量作为分配器, 并在init函数里面把那个数组的内存初始化给它. 想把数组的名字作为指针, 只需要调用.as_str()然后as转换为usize就可以了.
这样, main函数调用完heap的init之后就可以分配堆空间了.

答案中heap2则没使用buddy system, 而是使用自己的algorithm crate的bitmap_vector_allocator提供分配算法支持, 自己实现简单的分配算法也很简单.



## 封装
毕竟只要直接把那个LockedHeap的init函数调用了, 就能随便用堆了, 确实, 如果只是为了拓展内存空间, 直接定义一个开始位置, 然后init函数改下就好了.
但是内存地址空间确实是非常常用的东西. 无论是分页内存管理还是mmio, 之后肯定会大量用到内存地址的. 因此这里封装出了地址类.
1. 封装地址类和页号类
1. 抽象实现From trait在地址和页号间相互转换
1. 抽象实现地址转页号的向上/下取整
1. 抽象实现和usize的加减输出操作.


## 改进: 按页的物理内存管理 + frame tracker
硬编码可用物理地址
设置全局变量KERNEL_END_ADDRESS, PhysicalAddress类型.
MEMORY_START_ADDRESS, MEMORY_END_ADDRESS, KERNEL_HEAP_SIZE同理

为什么KERNEL_END_ADDRESS是lazy_static?
因为不用会报错"pointer-to-integer cast" needs an rfc before being allowed inside constants

frame tracker 作为物理页面的智能指针, 继承PhysicalPageNumber, 实现Drop.
frame模块除了frame_tracker的实现, 同时承载分配相关的实现: allocator.rs 对分配算法进行包装, 对接frame tracker和分配. 分配算法实现Allocator trait(一次分配回收一个元素的index), 包装后提供初始化创建(起始物理页号和页数量), 分配一个页(frame tracker), frame tracker析构的时候自动调用回收.
由于是简单的一次分配一个元素, 而且可以是离散的, 简单地使用一个栈进行分配. 创建StackedAllocator, 在allocator.rs中作为AllocImpl, 就会使用这个算法了.

最后还要在memory中新建mod range. 提供对内存区域的支持. 它在基础的core::ops::range基础上增加相互转换, 和len函数, 和迭代, 重叠检测, 虚拟物理相互转换, 取下标, 包含检测的功能.

frame tracker创建的时候不会自动申请页面, 因此想要获得frame tracker需要通过allocator分配.

思考题有些神奇, 不把申请到的页面的所有权传到外面, 就会死锁.

## 使用自己的堆分配算法
堆分配算法对外的接口一个是`#[global_allocator]`标注, 一个是init函数.
使用的是bitmap标记空闲, 以字节为单位, 查找时按照对齐要求的倍数顺序查找(作为内存的开头), 直到遇到了空闲处.
只标记4096字节, 最多只能管理4K的内存. 这里的实现也是对usize偏移index做分配, 不考虑基地址.
接着把旧的heap.rs重命名为heap_old.rs 新建heap.rs对算法封装. 为了global_allocator要实现alloc::alloc::GlobalAlloc这个trait. 两个函数, 传入的是core::alloc::Layout, 并且需要处理指针类型 *mut u8. 直接整个实现都是unsafe的. Heap全局变量在算法的基础上包了一层Option, 再包UnsafeCell.
UnsafeCell需要get再as_mut, Option就unwarp, 就可以得到内部的VectorAllocatorImpl调用alloc/dealloc函数.

Option的作用是提供默认值?? 初始化之前, Option里是None, 初始化函数使用replace函数替换成实例之后才能分配, 否则会在unwrap的时候panic

完成之后, 将堆测试的两个循环数量从10000减少到100才能正常通过测试. 看来还是不太好用, 切换回之前的buddy system算法吧

下面这句话的下划线是什么意思??
```rust
let offset = ptr as usize - &HEAP_SPACE as *const _ as usize;
```

## 问题
context.rs中给context结构体实现了Default trait, 使用全零. 这里不实现会怎么样

下面这个impl是什么意思?
```rust
pub fn iter(&self) -> impl Iterator<Item = T> {
    (self.start.into()..self.end.into()).map(T::from)
}
```

## 用到的其他知识点
rustdoc的注释写法里`//!`是什么意思??

print trait的实现
抽象实现的时候的宏的用法, 宏参数的几个类型

https://doc.rust-lang.org/reference/visibility-and-privacy.html
pub(super) 对父模块公开