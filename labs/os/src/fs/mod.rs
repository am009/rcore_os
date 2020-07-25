
mod config;
mod inode_ext;
pub mod stdin;
pub mod stdout;

use lazy_static::lazy_static;
use alloc::{sync::Arc, vec::Vec};
use rcore_fs_sfs::SimpleFileSystem;
use crate::drivers::{
    block::BlockDevice,
    driver::{DeviceType, DRIVERS},
};

pub use config::*;
pub use inode_ext::INodeExt;
pub use rcore_fs::{dev::block_cache::BlockCache, vfs::*};

lazy_static! {
    pub static ref ROOT_INODE: Arc<dyn INode> = {
        for driver in DRIVERS.read().iter() {
            if driver.device_type() == DeviceType::Block {
                let device = BlockDevice(driver.clone());
                let device_with_cache = Arc::new(BlockCache::new(device, BLOCK_CACHE_CAPACITY));
                return SimpleFileSystem::open(device_with_cache)
                    .expect("failed to open SFS")
                    .root_inode();    
            }
        }
        panic!("failed to load fs")
    };
}

pub fn init() {
    // ROOT_INODE.ls();
    ls("/");
    println!("mod fs initialized.");
}

pub fn ls(path: &str) {
    let mut id = 0;
    let dir = ROOT_INODE.lookup(path).unwrap();
    print!("files in {}: \n  ", path);
    while let Ok(name) = dir.get_entry(id) {
        id += 1;
        print!("{} ", name);
    }
    print!("\n");
}