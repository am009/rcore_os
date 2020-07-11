
// 笨办法学C 练习11, while循环和bool表达式.
fn main() {
    let mut args = std::env::args();
    let argc = std::env::args().count() as i32;
    let mut i: i32 = 0;
    while i < argc {
        println!("arg {}: {}", i, args.next().unwrap());
        i = i + 1;
    }

    let mut states: [&str; 4] = [ "California", "Oregon",
    "Washington", "Texas" ];
    
    let num_states = 4;
    i = 0;
    while i < num_states {
        println!("state {}: {}", i, states[i as usize]);
        i = i + 1;
    }

    // 附加题1
    args = std::env::args();
    i = argc;
    while i > 0 {
        println!("arg {}: {}", argc - i, args.next().unwrap());
        i = i - 1;
    }
    // 附加题2
    let args: Vec<String> = std::env::args().collect();
    i = 0;
    while i < num_states && i < argc {
        states[i as usize] = &args[i as usize];
        i = i + 1;
    }
    
    i = 0;
    while i < num_states {
        println!("state {}: {}", i, states[i as usize]);
        i = i + 1;
    }
}