
// 笨办法学C练习18: 函数指针
// 写起来真是艰难啊

fn bubble_sort(nums: & Vec<i32>, cmp: &dyn Fn(i32, i32) -> bool) -> Vec<i32> {
    let mut target = nums.clone();
    // println!("hey");
    for i in 0..target.len() {
        for j in 0..target.len() {
            if cmp(target[i], target[j]) {
                target.swap(i, j)
            }
        }
    }
    target
}

fn test_sorting(nums: & Vec<i32>, cmp: &dyn Fn(i32, i32) -> bool) -> Vec<i32> {
    let sorted_num = bubble_sort(nums, cmp);
    println!("{:?}", sorted_num);
    sorted_num
}

fn main () -> Result<(), Box<dyn std::error::Error>>  {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        println!("usage: function 4 3 1 5 6");
        return Ok(());
    }
    println!("{:?}", args);
    let mut nums: Vec<i32> = Vec::new();
    let mut iter = args.iter();
    iter.next();
    for arg in iter {
        nums.push(arg.parse()?);
    }
    println!("input: {:?}", nums);

    test_sorting(&nums, &|x, y| { x > y });
    test_sorting(&nums, &|x, y| { x < y });
    test_sorting(&nums, &|x, y| { if x == 0 || y == 0 { false } else { (x % y) != 0 } });

    return Ok(());
}