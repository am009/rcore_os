
fn print_arguments(argv: Vec<String>) {
    for arg in argv {
        print_letters(&arg);
    }
}

fn print_letters(arg: &str) {
    for c in arg.chars() {
        if can_print_it(&c) {
            println!("'{}' == {}", c, c as u8);
        }
    }
}

fn can_print_it(ch: &char) -> bool {
    ch.is_alphanumeric() || ch.is_whitespace()
}

fn main() {
    print_arguments(std::env::args().collect())
}