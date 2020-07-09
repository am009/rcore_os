

// 笨办法学C练习10, 字符串数组和循环
// 遍历String有两种方法, 分别是bytes和chars
// 由于utf-8编码的原因, 取对应位置的码点已经不是常数基本的运算了, 因此标准库中没有实现下标运算.
fn main() {
    let mut vec = Vec::new();

    let args: Vec<String> = std::env::args().collect();
    // let args: &[&str] = std::env::args().collect(); // error
    println!("{:?}", args);
    for i in std::env::args() {
        print!("{} ", i);
        vec.push(i);
    }
    
    println!("{:?}", vec);
    // let arr: &[str]; // error: doesn't have a size known at compile-time
    // array内的需要是大小确定的元素
    // array内元素类型相同, 
    // tuple内元素不需要类型相同, 更像是匿名结构体.
    let arr: &[&str] = &["California", "Oregon",
                        "Washington", "Texas"];
    println!("{:?}", arr);

    for i in 0..4 {
        println!("{}", arr[i]);
    }
    // 数组越界后会运行时panic
    // for i in 0..=4 { // panic
    //     println!("{}", arr[i]);
    // }
    
    println!("---------------------");
    let mut hello = String::from("Hello, rust string!😂");
    hello.push('😀');
    hello.push_str("haha");
    println!("{}", hello);
    // 从u8数组解出utf-8
    let mut sparkle_heart = String::from_utf8(vec![240, 159, 146, 150]).unwrap();
    println!("{}", sparkle_heart);
    // 转换回utf-8
    println!("{:?}", sparkle_heart.as_bytes());
    // 使用chars和nth来代替下标
    println!("{}", hello.chars().nth_back(4).unwrap());
    // String 的长度是以字节为单位的.
    println!("{}", sparkle_heart.len());
    // as_str() 和*解引用一个string是一样的, 都可以达到将string转换为str的效果.
    // str和String最大的区别在于String可以自由变化长度.
    let hello_str = &mut *hello;
    hello_str.make_ascii_uppercase();
    
    println!("{}", hello_str);
    println!("{}", hello);
    // string 有pop和push方法,弹出字符
    sparkle_heart.push('💖');
    println!("{}", sparkle_heart.pop().unwrap());
    // remove方法移除字符的同时会返回它, 这是一个O(n)的操作 insert同理
    println!("{}", sparkle_heart.remove(0));
    println!("{}", sparkle_heart == "");
    sparkle_heart.insert(0, '💖');
    // 删除和插入一段str
    hello.replace_range(7..=10, "");
    hello.insert_str(7, "world");
    println!("{}", hello);
    // 更加丰富多彩的是str的方法, 由于实现了deref的trait, String也可以直接使用str的方法.
    // 之前的chars()和bytes()就是str的方法
    assert_eq!(hello.contains("world"), true);
    assert_eq!(hello.ends_with("AHA"), true);
    assert_eq!(hello.starts_with("HELLO"), true);
    // 上面这些传入的都是 Pattern, 目前还没实现完全, 
    // 现在可以使用简单的字符串, 字符和FnMut(char) -> bool作为pattern
    // 这个传入的函数有点有限状态自动机的味道.
    // https://doc.rust-lang.org/std/str/pattern/index.html
    println!("{:?}", hello.find("."));
    // trim_end_matches, trim_start_matches系列方法和strip_prefix strip_suffix系列方法的区别是
    // 前者会反复移除, 后者只会移除一次
    println!("{:?}", "hello world......".trim_end_matches(|c: char| c == '.'));
    // split 系列方法返回的是iterator, 使用collect可以转化为Vec等
    let mut l: Vec<&str> = "  hel lo .. ".split_ascii_whitespace().collect();
    println!("{:?}", l);
    // split_inclusive 则会包含分隔符, 这个api还不稳定.
    // l = "  hel lo .. ".split_inclusive(' ').collect();
    // println!("{:?}", l);
}
