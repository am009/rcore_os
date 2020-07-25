
use super::*;
use alloc::collections::VecDeque;
use spin::Mutex;
use crate::kernel::Condvar;
use core::any::Any;

lazy_static! {
    pub static ref STDIN: Arc<Stdin> = Default::default();
}

#[derive(Default)]
pub struct Stdin {
    buffer: Mutex<VecDeque<u8>>,
    condvar: Condvar,
}

impl INode for Stdin {
    fn read_at(&self, offset: usize, buf: &mut [u8]) -> Result<usize> {
        if offset != 0 {
            Err(FsError::NotSupported)
        } else if self.buffer.lock().len() == 0 {
            // 条件变量会自动放开mutex ??
            self.condvar.wait();
            Ok(0)
        } else {
            // 对buf的每个字节遍历, 不断从STDIN取字符, 当字符不够返回, buf大小不足也返回
            let mut stdin_buffer = self.buffer.lock();
            for (i, byte) in buf.iter_mut().enumerate() {
                if let Some(b) = stdin_buffer.pop_front() {
                    *byte = b;
                } else {
                    // stdin 没有了
                    return Ok(i);
                }
            }
            // buf装不完
            Ok(buf.len())
        }
    }

    fn write_at(&self, _offset: usize, _buf: &[u8]) -> Result<usize> {
        Err(FsError::NotSupported)
    }

    fn poll(&self) -> Result<PollStatus> {
        Err(FsError::NotSupported)
    }

    /// This is used to implement dynamics cast.
    /// Simply return self in the implement of the function.
    fn as_any_ref(&self) -> &dyn Any {
        self
    }
}

impl Stdin {
    /// 向缓冲区插入一个字符，然后唤起一个线程
    pub fn push(&self, c: u8) {
        self.buffer.lock().push_back(c);
        self.condvar.notify_one();
    }
}