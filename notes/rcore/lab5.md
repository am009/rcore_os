# lab5 设备驱动与文件系统

## 综述
1. 提供对ELF文件用户程序的解析
2. 提供对磁盘映像的支持
3. 实现系统调用支持
4. 支持文件描述符

## 相关修改

0. entry.asm 增加映射低位页表
1. main函数增加参数, 调用drivers的初始化函数
2. 新建drivers文件夹, 在mod.rs中增加init函数, 调用设备树子模块的init函数
3. drivers/device_tree.rs 增加对device_tree这个库的依赖. init函数调用dtb遍历库, walk函数负责遍历生成的树.
4. 修改makefile, 增加qemu的启动参数 增加TEST_IMG变量指向之后的磁盘镜像
5. 增加drivers/bus文件夹并增加子模块virtio_mmio. 

## entry.asm 
这里没想到增加了新的页表. 花了我差不多一个小时的时间去debug
当parse dtb, 得到磁盘的Header的时候, 这个header的位置在0x1000_8000, 位于低地址, 而我们此时还处于初始化的状态, 映射还是巨页映射, 只映射了0x8000 0000开头的1GB, (1GB是0x4000 0000). 
更加恐怖的是, 此时的sscratch都没有值, 中断都无法正常进行. 
当前的设计是中断一定切换栈, 每次中断的时候都是先交换sscratch和sp, 然后保存真正的原来的sp到栈上. 离开的时候先把sp弹出后的位置放到sscratch. 而第一次放入sscratch就是运行线程时假装中断返回的时候.
因此, 我在debug的时候, 第一次页访问异常, 进入中断的时候, 从sscratch中取来了(似乎是-1)未知的值, 作为内核栈放到sp, 然后在保存第一个寄存器的时候发生了第二次页访问异常. 这时又把sscratch和sp交换, 得到了正常的sp值, 反而正常处理了, 但是报错的地址好像是0xffff_ffff_ffff_ffec.
我在main函数加入了把kernel stack放到sscratch的汇编才正常得到了中断发生错误的地址.

## drivers模块

driver模块目前主要负责文件系统的driver. 对外暴露的接口是driver模块中的driver trait, 和DRIVERS这个组织各种驱动的数据结构.

模块的初始化函数在mod.rs中, 参数传入dtb地址, 负责调用解析设备树的device_tree::init, device_tree的walk函数则一边遍历, 一边判断是不是想要的设备, 这里单指对应磁盘的块设备, 找到则把这个节点传给对应的总线协议驱动程序, 这里是bus/virtio_mmio. 本次实验中找到块设备的节点后, 把节点里reg字段拿出, 转换为virtio_drivers(库)::VirtIOHeader类型, 就传给驱动程序的包装virtio_blk, virtio_blk::VirtIOBlkDriver内部包装的是mutex包装的virtio_drivers(库)::VirtIOBlk, 对外实现了driver的trait.

之后在fs中才会用到BlockDevice, 它包装driver, 实现rcore_fs的BlockDevice trait从而传入给SimpleFileSystem::open函数

层次关系:
virtio_drivers(库)::VirtIOBlk --包装--> VirtIOBlkDriver(实现Driver) --包装--> BlockDevice(实现rcore_fs的BlockDevice)

### driver.rs
这个模块包含了驱动接口driver trait, 描述驱动类型的DeviceType, 和lazy_static的DRIVERS.

Driver 这个trait, 使用时根据device_type返回的DeviceType, 来调用对应的方法, 现在这个trait中只有块设备相关的方法, 提供了用unimplemented宏表明没有实现这个方法的默认实现. 实现新的设备驱动的时候, 只需要加入新的方法即可.

DRIVERS保存所有驱动的数据结构, 方便获取驱动. 驱动是dyn Driver类型, 首先用Arc实现共享, 再通过Vec保存, 再加上读写锁保证多线程安全.

### device_tree.rs

device tree blob 确实是个标准. 其中头部的字段其实很多, 包括了dtb的版本等等. 我们现在只读取了前两个字段. 第二个字段size确实是包含整个dtb的大小的, 包括头部.
https://www.devicetree.org/specifications/

这里init函数首先校验头部, 得到size, 再把整个dtb作为u8数组传入DeviceTree的crate中, 并且让遍历的walk函数递归遍历.
而walk函数则只是搜索compatiable字段为virtio,mmio的节点, 把节点传入virtio_probe进行初始化

### bus/virtio_mmio.rs
将传入的dtb节点的reg字段转为VirtIOHeader传入驱动程序进行初始化.

这里会遇到不少verify不对的设备, 因此如果verify函数调用失败或者没有reg字段就直接返回. 这里verify的时候就会访问1000_xxxx开头的低地址.

此外, 暴露了virtio_dma_dealloc, virtio_phys_to_virt, virtio_virt_to_phys这三个extern "C"且no_mangle的函数. 而且没有在我们代码中其他地方被调用. 这说明是库函数在C语言中或者汇编中调用了这两个函数. 根据名字看可能是virtio库.

### block模块和 block/virtio_blk.rs
block模块的mod.rs里提供了对接驱动与文件系统的VirtIOBlkDriver包装. 而模块内部则是保存的块设备的Driver实现.

BlockDevice的实现主要是将Driver的返回bool值的read_block/write_block 函数转换成返回Result<()>的read_at/write_at函数, 另外实现假的sync函数

VirtIOBlkDriver的实现就是调用内部的Driver的read_block/write_block函数, 把返回的Result再用is_ok转成bool.

VirtIOBlkDriver则需要实现read_block/write_block的Driver接口, 另外给解析Node的virtio_mmio.rs:virtio_probe函数一个创建设备的add_driver函数. add_driver函数把header传给VirtIOBlk::new得到包装的内部驱动, 再把驱动包装上刚刚实现的VirtIOBlkDriver加入DRIVERS列表.

## fs模块

模块的mod.rs 提供了lazy_static的全局变量ROOT_INODE, 初始化的时候获取第一个Block类型的driver, 用BlockDevice包装后传入SimpleFileSystem::open()函数, 返回值赋给ROOT_INODE. 也许是SimpleFileSystem实现了对INode的deref, 在ROOT_INODE上可以调用到inode_ext拓展INode实现的方法

还有init函数, 负责作为测试, main函数初始化的时候使用ls方法测试我们文件系统的功能.

### inode_ext.rs
impl INodeExt for dyn INode 通过这种方法为已有的Inode类型增加功能. 额外实现了 ls这个直接打印而不返回值的函数, 和readall函数, 把所有内容读到`Vec<u8>`并返回.

## elf相关代码

ELF文件也可以看作是一个地址空间. 因为它定义了各个段的映射关系.
MemorySet中增加根据ELF文件创建的from_elf函数, 它遍历elf文件的每个段, 根据大小和权限映射每个段.

首先在program_header中对每个为load类型的段, 读取开始地址和大小和数据和权限之后进行映射

process中也加入from_elf函数, 主要是调用MemorySet中的from_elf函数.

为什么这里获取的data是SegmentData::Undefined类型??

## 其他疑问

virtio_blk.rs 中的BLOCK_SIZE_LOG2成员是做什么的??


## ucore sfs学习

文件夹inode -> 文件entry -> 文件inode -> 文件数据