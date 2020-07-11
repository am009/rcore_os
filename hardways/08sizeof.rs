// 笨办法学C练习8 大小和数组

// 在rust里面, 想要获取大小使用std::mem::size_of
// https://doc.rust-lang.org/std/mem/fn.size_of.html
// 而且这里还详细讲了#[repr(C)] 的结构体的大小计算

// 想要获取引用指向的结构体的大小可以通过std::mem::size_of_val
// 这种方法可以运行时确定大小
// https://doc.rust-lang.org/std/mem/fn.size_of_val.html

fn main () {
    let areas: [i32; 5] = [10, 12, 13, 14, 20];
    let name: &str = "Zed";

    // let full_name = [90 as u8, 101, 100, 32, 65, 46, 32, 83, 104, 97, 119];
    // println!("{:?}", std::str::from_utf8(&full_name).unwrap());
    let full_name = ['Z', 'e', 'd', ' ', 'A', '.', ' ',
                        'S', 'h', 'a', 'w'];

    print!("The size of an int: {}\n", std::mem::size_of::<i32>());
    print!("The size of areas (i32[]): {}\n",
            std::mem::size_of_val(&areas));
    print!("The number of ints in areas: {}\n",
    std::mem::size_of_val(&areas) / std::mem::size_of::<i32>());
    print!("The first area is {}, the 2nd {}.\n",
            areas[0], areas[1]);

    print!("The size of a char: {}\n", std::mem::size_of::<char>());
    print!("The size of name (char[]): {}\n",
    std::mem::size_of_val(&name));
    print!("The number of chars: {}\n",
    std::mem::size_of_val(&name) / std::mem::size_of::<char>());

    print!("The size of full_name (str): {}\n",
    std::mem::size_of_val(&full_name));
    print!("The number of chars: {}\n",
    std::mem::size_of_val(&full_name) / std::mem::size_of::<char>());

    print!("name=\"{}\" and full_name=\"{}\"\n",
            name, full_name.iter().collect::<String>());
}