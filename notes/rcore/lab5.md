# lab5 设备驱动与文件系统

## 综述
1. 提供对ELF文件用户程序的解析
2. 提供对磁盘映像的支持
3. 实现系统调用支持
4. 支持文件描述符

内核线程加载, 


## 相关修改

0. entry.asm 增加映射低位页表
1. main函数增加参数, 调用drivers的初始化函数
2. 新建drivers文件夹, 在mod.rs中增加init函数, 调用设备树子模块的init函数
3. drivers/device_tree.rs 增加对device_tree这个库的依赖. init函数调用dtb遍历库, walk函数负责遍历生成的树.
4. 修改makefile, 增加qemu的启动参数 增加TEST_IMG变量指向之后的磁盘镜像
5. 增加drivers/bus文件夹并增加子模块virtio_mmio. 

## entry.asm 
这里没想到增加了新的页表. 花了我差不多一个小时的时间去debug

## device_tree.rs

device tree blob 确实是个标准. 其中头部的字段其实很多, 包括了dtb的版本等等. 我们现在只读取了前两个字段. 第二个字段size确实是包含整个dtb的大小的, 包括头部.
https://www.devicetree.org/specifications/

这里init函数首先校验头部, 得到size, 再把整个dtb作为u8数组传入DeviceTree的crate中, 并且让遍历的walk函数递归遍历.
而walk函数则只是搜索compatiable字段为virtio,mmio的节点, 把节点传入virtio_probe进行初始化

## ucore sfs学习

文件夹inode -> 文件entry -> 文件inode -> 文件数据