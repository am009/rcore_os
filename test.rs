fn main() {
    let mut s = std::string::String::from("123123");
    let mut s2 = &mut (*(&mut s));
    s2.insert_str(0, "Hello ");
    println!("{}", s);
    // println!("{}", s2);
}