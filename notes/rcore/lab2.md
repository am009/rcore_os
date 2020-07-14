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

直接分配u8 static数组,数组名字指向的就是对应的空间.
添加buddy system这个包. spin和lazy_static也顺便加上.
创建memory文件夹作为新的mod, 创建init函数被main调用. 创建一个HEAP全局变量作为分配器, 并在init函数里面把那个数组的内存初始化给它. 想把数组的名字作为指针, 只需要调用.as_str()然后as转换为usize就可以了.
这样, main函数调用完heap的init之后就可以分配堆空间了.

## 问题
context.rs中给context结构体实现了Default trait, 使用全零. 这里不实习会怎么样
单独impl Debug具体怎么做??