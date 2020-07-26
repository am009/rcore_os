# 挑战实验题

## 按Ctrl-C杀死当前线程
在处理外部中断读取键盘字符时, 加入打印的函数, 发现当按下Ctrl-C会产生ascii编码为3的控制字符, 在此处判断如果字符等于3的话就杀死当前线程(和fault函数一样).

## fork的实现

写得比较晚还是有好处的. 现在的实现直接既复制进程又复制线程.
目前还没有写时复制机制, 打算把低位地址全部复制了去.

1. 为thread增加clone_thread方法. Context实现了Clone, 就直接调用. 关键在于process的复制
2. 为process实现clone的trait, 关键在于memory_set. descriptor则遍历调用clone复制(STDIN/STDOUT都是arc的)
3. 为memory_set实现clone的trait, 每个用户进程都是基于内核的映射的, 先建立内核映射, 再依次复制其他映射(segments实现了Eq), 可以发现栈和其他映射都是Framed类型的.
4. 关键在于内存地址的复制, 可以直接调用memory_set的add_segment, 问题是初始数据. 

如果页表就是当前页表, 可以直接根据虚拟地址复制.
如果页表不一定是当前页表, 那就需要根据页表找到对应的物理地址, 并且(利用线性偏移)访问物理地址获取数据. 
这查找的函数, 有类似的lookup函数, 但是也是需要当前页表, 是satp寄存器里的

### 实现支持非当前线程的clone

首先对mappings增加lookup_self函数, 在lookup的基础上修改, 只不过使用的不少satp寄存器, 而是自己保存的页目录表的页号来查找.

memory_set增加clone_segment函数, 类似add_segmen函数, 但是调用的是mapping的map_clone函数(而不是原有的map函数)

最后mappings增加map_clone函数, 在map的基础上修改, 不过不是从传入的data获得数据, 而是从传入的另外一个mapping调用lookup_self获取物理地址, 用线性偏移访问得到数据.

### 测试
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
