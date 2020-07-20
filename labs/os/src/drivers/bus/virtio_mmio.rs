
use super::super::block::virtio_blk;
use crate::memory::{
    frame::{FrameTracker, FRAME_ALLOCATOR},
    mapping::Mapping,
    config::PAGE_SIZE,
    address::{
        PhysicalAddress, VirtualAddress,
    }
};
use alloc::collections::btree_map::BTreeMap;
use device_tree::{util::SliceRead, Node};
use lazy_static::lazy_static;
use spin::RwLock;
use virtio_drivers::{DeviceType, VirtIOHeader};

/// 找类型为Block的
pub fn virtio_probe(node: &Node) {
    let reg = match node.prop_raw("reg") {
        Some(reg) => reg,
        _ => return,
    };
    let pa = PhysicalAddress(reg.as_slice().read_be_u64(0).unwrap() as usize);
    // println!("device header address at {:x?}", pa);
    let va = VirtualAddress::from(pa);
    // println!("device header va at {:x?}", va);
    let header = unsafe { &mut *(va.0 as *mut VirtIOHeader) };
    // pub fn verify(&self) -> bool {
    //     self.magic.read() == 0x7472_6976 && self.version.read() == 1 && self.device_id.read() != 0
    // }
    if !header.verify() {
        // println!("virtio_probe: header varification failed.");
        // println!("{:x?}", header);
        return;
    }
    match header.device_type() {
        DeviceType::Block => virtio_blk::add_driver(header),
        device => println!("unrecognized virtio device: {:?}", device),
    }
}

lazy_static! {
    pub static ref TRACKERS: RwLock<BTreeMap<PhysicalAddress, FrameTracker>> = 
        RwLock::new(BTreeMap::new());
}

/// 试图分配连续的物理内存
#[no_mangle]
extern "C" fn virtio_dma_alloc(pages: usize) -> PhysicalAddress {
    let mut pa: PhysicalAddress = Default::default();
    let mut last: PhysicalAddress = Default::default();
    for i in 0..pages {
        let tracker: FrameTracker = FRAME_ALLOCATOR.lock().alloc().unwrap();
        if i == 0 {
            pa = tracker.address();
        } else {
            assert_eq!(last + PAGE_SIZE, tracker.address());
        }
        last = tracker.address();
        TRACKERS.write().insert(last, tracker);
    }
    pa
}

#[no_mangle]
extern "C" fn virtio_dma_dealloc(pa: PhysicalAddress, pages: usize) -> i32 {
    for i in 0..pages {
        TRACKERS.write().remove(&(pa + i * PAGE_SIZE));
    }
    0
}

#[no_mangle]
extern "C" fn virtio_phys_to_virt(pa: PhysicalAddress) -> VirtualAddress {
    // println!("looking up pa {:x?}", pa);
    VirtualAddress::from(pa)
}

#[no_mangle]
extern "C" fn virtio_virt_to_phys(va: VirtualAddress) -> PhysicalAddress {
    // println!("looking up va {:x?}", va);
    Mapping::lookup(va).unwrap()
}