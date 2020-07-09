use std::io::prelude::*;

// 笨办法学Ruby15, 读取文件
// Read 这个trait是在io里给File实现的, 不引入就会保存没有read_to_string这样的错
// Read trait, 需要实现函数read(), 提供了函数read_to_end() read_to_string read_exact这几个函数
// https://doc.rust-lang.org/std/io/trait.Read.html
// stdin是一个特例, 它自己实现了read_line(), stdin的read_line函数指向了BufRead函数
// 使用let f = std::io::BufReader::new(f)套一层, 这样就能获得BufRead的功能了.
// 能获得read_until/ read_line, split, lines这几个函数

fn main () -> Result<(), Box<dyn std::error::Error>>  {
    let mut argv: Vec<String> = std::env::args().collect();
    if argv.len() < 2 {
        // println!("usage file.exe path")
        argv.push("README.md".to_string())
    }

    println!("Here's your file: {}", argv[1].trim());
    let mut file = std::fs::File::open(&std::path::Path::new(argv[1].trim()))?;

    let mut buf = String::new();
    file.read_to_string(&mut buf)?;
    println!("{}",buf);

    print!("type your file path again:\n> ");
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();
    stdout.flush()?;
    
    let mut filepath = String::new();
    stdin.read_line(&mut filepath)?;
    // println!("{}",filepath.trim());
    
    // 手动trim
    if let Some('\n')=filepath.chars().next_back() {
        println!("Trimmed \\n");
        filepath.pop();
    }
    if let Some('\r')=filepath.chars().next_back() {
        println!("Trimmed \\r");
        filepath.pop();
    }
    // 默认值
    if filepath == "" {
        filepath.push_str("README.md")
    }

    file = std::fs::File::open(&std::path::Path::new(&filepath))?;
    
    buf = String::new();
    file.read_to_string(&mut buf)?;
    println!("{}",buf);

    println!("---------------------");
    

    Ok(())
}