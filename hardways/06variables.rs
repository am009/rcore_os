// 笨办法学C中, 这一节是为了
// 学习各种各样的变量类型的
// 非常简单
fn main() {
    let distance: i32 = 100;
    let power: f32 = 2.345f32;
    let super_power: f64 = 56789.4532;
    let initial: char = 'A';
    let first_name: &str = "Zed";
    let last_name: &str = "Shaw";

    print!("You are {} miles away.\n", distance);
    print!("You have {} levels of power.\n", power);
    print!("You have {} awesome super powers.\n", super_power);
    print!("I have an initial {}.\n", initial);
    print!("I have a first name {}.\n", first_name);
    print!("I have a last name {}.\n", last_name);
    print!("My whole name is {} {}. {}.\n",
            first_name, initial, last_name);
}