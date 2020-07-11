
// 笨办法学C 练习13 switch语句
fn main() {
    let mut args = std::env::args();
    let argc = std::env::args().count() as i32;

    if argc != 2 {
        println!("ERROR: You need one argument.");
        return
    }

    args.next();
    let mut i = 0;
    for letter in args.next().unwrap().chars() {
        match letter {
            'a' | 'A' => println!("{}: 'A'", i),
            'e' | 'E' => println!("{}: 'E'", i),
            'i' | 'I' => println!("{}: 'I'", i),
            'o' | 'O' => println!("{}: 'O'", i),
            'u' | 'U' => println!("{}: 'U'", i),
            'y' | 'Y' => { if i > 2 { println!("{}: 'Y'", i) } },
            other => println!("{}: {} is not a vowel", i, other)
        }
        i = i + 1;
    }
}