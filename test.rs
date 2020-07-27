

fn main() {
    let umax = usize::MAX;
    let imax = isize::MAX;
    println!("umax: {:x}, imax: {:x}, uasi: {}", umax, imax, umax as isize);
    let ret1 = (1usize as isize).cmp(&((umax - 1) as isize));
    let ret2 = 1usize.wrapping_sub(umax - 1) as isize;
    let ret3 = (umax - 1).wrapping_sub(1) as isize;
    println!("{:?}, {:x}, {}", ret1, ret2, ret3);
    
    // let ret1 = (self.pass as isize).cmp(&(other.pass as isize)).reverse();
    // let diff = self.pass.wrapping_sub(other.pass) as isize;
    // let ret2 = if diff < 0 { Ordering::Greater } else if diff == 0 { Ordering::Less } else { Ordering::Equal };
    // assert_eq!(ret1, ret2);
}