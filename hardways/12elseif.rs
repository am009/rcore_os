
// 笨办法学C 练习12, If, Else If, Else

fn main() {
    let args = std::env::args();
    let argc = std::env::args().count() as i32;

    if argc == 1 {
        println!("You only have one argument. You suck.");
    } else if argc > 1 && argc < 4 {
        println!("Here's your arguments:");

        for arg in args {
            print!("{} ", arg);
        }
        print!("\n");
    } else {
        println!("You have too many arguments. You suck.");
    }
}