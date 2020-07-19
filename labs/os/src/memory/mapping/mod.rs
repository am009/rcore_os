//! 对内存映射的封装
//!
//! 每个线程保存一个[`Mapping`], 记录了所有的[`Sengment`]
//! drop的时候安全释放所有资源

mod mapping;
mod memory_set;
mod page_table;
mod page_table_entry;
mod segment;

pub use mapping::Mapping;
pub use memory_set::MemorySet;
pub use page_table::{PageTable, PageTableTracker};
pub use page_table_entry::{Flags, PageTableEntry};
pub use segment::{MapType, Segment};
