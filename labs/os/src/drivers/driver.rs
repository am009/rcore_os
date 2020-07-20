//! 驱动接口的定义
//!
//! 目前接口中只支持块设备类型

use alloc::{sync::Arc, vec::Vec};
use lazy_static::lazy_static;
use spin::RwLock;

#[derive(Debug, Eq, PartialEq)]
pub enum DeviceType {
    Block,
}

pub trait Driver: Send + Sync {
    fn device_type(&self) -> DeviceType;
    fn read_block(&self, _block_id: usize, _buf: &mut [u8]) -> bool {
        unimplemented!("not a block driver")
    }
    fn write_block(&self, _block_id: usize, _buf: &[u8]) -> bool {
        unimplemented!("not a block driver")
    }
}

lazy_static!{
    pub static ref DRIVERS: RwLock<Vec<Arc<dyn Driver>>> = RwLock::new(Vec::new());
}