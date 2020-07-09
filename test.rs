fn main() {
    let mut s = std::string::String::from("123123");
    let s2 = &mut s;
    let mut a = *s2;
    // println!("{}", s2);
}