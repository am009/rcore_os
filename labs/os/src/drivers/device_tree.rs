


use super::bus::virtio_mmio::virtio_probe;
use crate::memory::address::VirtualAddress;
use core::slice;
use device_tree::{DeviceTree, Node};

const DEVICE_TREE_MAGIC: u32 = 0xd00d_feed;

fn walk(node: &Node) {
    // println!("{:?}", node);
    if let Ok(compatiable) = node.prop_str("compatible") {
        if compatiable == "virtio,mmio" {
            // println!("find compatible device type: {}", compatiable);
            virtio_probe(node);
            // println!("virtio_probe finished.");
        }
    }
    for child in node.children.iter() {
        walk(child);
    }
}

struct DtbHeader {
    magic: u32,
    size: u32,
}

pub fn init(dtb_va: VirtualAddress) {
    let header = unsafe { &*(dtb_va.0 as *const DtbHeader) };
    let magic = u32::from_be(header.magic);
    if magic == DEVICE_TREE_MAGIC {
        let size = u32::from_be(header.size);
        let data = unsafe { slice::from_raw_parts(dtb_va.0 as *const u8, size as usize) };
        if let Ok(dt) = DeviceTree::load(data) {
            println!("walk dtb started.");
            walk(&dt.root);
        }
    } else {
        println!("dtb magic check failed!!!");
    }
}