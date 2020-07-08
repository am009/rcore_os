

// 笨办法学C练习10
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
    let arr: &[&str] = &["California", "Oregon",
                        "Washington", "Texas"];
    println!("{:?}", arr);

    for i in 0..4 {
        println!("{}", arr[i]);
    }
    // 数组越界后会panic
    // for i in 0..=4 { // panic
    //     println!("{}", arr[i]);
    // }
}
