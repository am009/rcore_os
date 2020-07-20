
use super::*;

pub trait INodeExt {
    fn ls(&self);
    /// 读取文件内容
    fn readall(&self) -> Result<Vec<u8>>;
}

impl INodeExt for dyn INode {
    fn ls(&self) {
        let mut id = 0;
        while let Ok(name) = self.get_entry(id) {
            println!("{}", name);
            id += 1;
        }
    }

    fn readall(&self) -> Result<Vec<u8>> {
        let size = self. metadata()?.size;
        let mut buffer = Vec::with_capacity(size);
        unsafe { buffer.set_len(size) };
        self.read_at(0, buffer.as_mut_slice())?;
        Ok(buffer)
    }
}