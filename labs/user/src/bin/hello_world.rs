#![no_std]

#![no_main]
#[macro_use]
extern crate user_lib;

#[no_mangle]
pub fn main() -> usize {
    println!("Hello from user space!!");
    0
}