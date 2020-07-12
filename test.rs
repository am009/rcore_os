fn choose<'a, 'b>(is_a: bool, a: &'a str, b: &'b str) -> Enum<'b, 'a> {
    if is_a {
        Enum::A(a)
    } else {
        Enum::B(b)
    }
}

#[derive(Debug)]
enum Enum<'a, 'b> {
    A(&'a str),
    B(&'b str)
}

fn main() {
    let mut s = std::string::String::from("123123");
    let s2 = &mut s;
    let mut a = *s2;
    // println!("{}", s2);
}