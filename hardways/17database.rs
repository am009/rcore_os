// 笨办法学C练习17, 堆和栈的内存分配
// 一个代码量突然变多的"简单"的数据库系统.
// 原来感觉挺简单的, 今天做起来才发现自己不理解Box的使用
// 卡了一个小时

// Box是智能指针, 它的思想是, 堆上分配的东西, 只能通过指针访问, 当指针失去作用(析构)的时候, 自然可以释放空间了
// 当需要传递堆上的数据的时候, 正好会传递指针(传递Box的所有权, 因此不会析构)

// rust 如果想要在堆上分配空间, 总感觉不够自由. 
// 这里的数据库系统, 确实符合Vec的使用场景, 但是确实不能自由控制堆上的空间
// 不如有时间看看Vec的源码, 如果我想要不使用Vec就实现这个的话, 可能会做了Vec底层相同的事情?
// 文档中说, 如果结构体是递归的, 则大小不固定, 不好分配空间, 
// 可以把这个递归的地方改用Box包装.

// 所有权的话, 似乎些不对的地方:
// C语言中可以直接给address数组整个分配空间, malloc完成后强制转换为数组指针.
// 对应到rust我凑出了这样的语句
// let mut a: &mut [i32] = (&mut Box::<[i32;5]>::new([0;5])) as &mut [i32;5];
// 这样的话, Box类型析构之后, a也就失效了, 这样做完全没有理解Box的用法
// 由于Box类型基本上可以当作内部包裹的类型的指针, 因此考虑直接使用box类型
// 考虑把boxed slice 当作普通slice使用
// let mut a: Box<[i32]> = Box::new([0;5]);

static MAX_DATA: i32 = 512;
static MAX_ROWS: i32 = 100;


// 而结构体的生命周期参数应对的是成员有引用时的情况.
// 指明结构体的生命周期和各个引用的生命周期的关系.
// 结构体实例的生命周期应短于或等于任意一个成员的生命周期。
// 如果成员先被析构, 那么结构体内部的对应引用就变成垂悬指针
struct Address<'a> {
    id: i32,
    set: i32,
    name: &'a str,
    email: &'a str
}

// struct Database {
//     rows: Box<[Address<'a>]>
// }

// struct Connection {
//     db: Box<Database>
// }

// impl Database {
//     fn open(path: std::path::Path, mode: char) -> Connection {
//         // C语言里返回的Connection里包含了堆上的Database指针
//         // 这里考虑返回Box<Database>
//     }
// }

// fn main () -> Result<(), Box<dyn std::error::Error>>  {
    
//     Ok(())
// }