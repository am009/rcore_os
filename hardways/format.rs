
//笨办法学C练习7

struct Color {
    red: u8,
    green: u8,
    blue: u8,
}

impl std::fmt::Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "RGB ({}, {}, {}) 0x{0:0>2x}{1:0>2x}{2:0>2x}", self.red, self.green, self.blue)
    }
}

// { name/position : [char|<^>]['+'/'-']['#']['0'][width$/num]['.' precision$/num][type] }
// https://doc.rust-lang.org/std/fmt/
// 格式化输出有五个相关宏, format!, print!, println!, eprint!, eprintln!
// 打印结构体(用{})需要实现fmt::Display trait
// 使用#时, 加的0x是在对齐和填充前的, 因此{:0>#4x}打印0会得到00x0
// 使用{:x?}可以让Debug trait打印出16进制的数字
// 没有指明的整数是i32, 浮点数是f64
fn main() {
    // Statements here are executed when the compiled binary is called
    let bugs: i64 = 100;
    let bug_rate: f64 = 1.2f64;

    print!("You have 0b{1:b} bugs at the imaginary rate of {0}.\n", bug_rate, bugs);
    let universe_of_defects: i32 = 1 * 1024 * 1024 * 1024;
    println!("The entire universe has 0x{defects:>0width$x} bugs.", defects=universe_of_defects, width=13);
    
    // no implementation for `i64 * f64` 浮点数和整数之间可以使用as转换类型.
    // let expected_bugs = bugs * bug_rate;
    let expected_bugs = bugs as f64 * bug_rate;
    // 如何打印变量的类型

    let part_of_universe = expected_bugs / universe_of_defects as f64;
    println!("That's only a {:e} protion of the universe.", part_of_universe);
    let nul_byte = '\0' as u8;
    let care_percentage = bugs * i64::from(nul_byte);
    println!("Which means you should care {}%.",
            care_percentage);
    for color in [
        Color { red: 128, green: 255, blue: 90 },
        Color { red: 0, green: 3, blue: 254 },
        Color { red: 0, green: 0, blue: 0 },
    ].iter() {
        // iter 返回的是每个元素的borrow
        println!("{}", *color);
    }
}
