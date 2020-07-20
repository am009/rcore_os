

use crate::memory::address::{PhysicalAddress, VirtualAddress};

pub mod block;
pub mod bus;
pub mod device_tree;
pub mod driver;

pub fn init(dtb_pa: PhysicalAddress) {
    let dtb_va = VirtualAddress::from(dtb_pa);
    device_tree::init(dtb_va);
    println!("mod driver initialized.");
}
