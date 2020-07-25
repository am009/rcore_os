
use spin::{Mutex, MutexGuard};

#[derive(Default)]
pub struct Lock<T>(pub(self) Mutex<T>);

// 使lockguard的生命周期和MutexGuard一样
pub struct LockGuard<'a, T> {
    guard: Option<MutexGuard<'a, T>>,
    sstatus: usize
}

impl<T> Lock<T> {
    pub fn new(obj: T) -> Self {
        Self(Mutex::new(obj))
    }
    // 约束使传出的LockGuard的生命周期和Lock的生命周期相同
    /// 关中断同时保存到sstatus变量
    pub fn get<'a>(&'a self) -> LockGuard<'a, T> {
        let sstatus: usize;
        unsafe {
            llvm_asm!("csrrci $0, sstatus, 1 << 1" : "=r"(sstatus) ::: "volatile");
        }
        LockGuard{
            guard: Some(self.0.lock()),
            sstatus
        }
    }
    pub fn is_locked(&self) -> bool {
        if let Some(_m) = self.0.try_lock() {
            false
        } else {
            true
        }
    }

    /// 不安全：获得不上锁的对象引用
    ///
    /// 这个只用于 [`PROCESSOR::run()`] 时使用, 因为那时候函数不会返回, 没机会执行关中断等操作
    ///
    /// [`PROCESSOR::run()`]: crate::process::processor::Processor::run
    pub unsafe fn unsafe_get(&self) -> &'static mut T {
        let addr = &mut *self.0.lock() as *mut T;
        &mut *addr
    }
}

/// 释放时，先释放内部的 MutexGuard，再恢复 sstatus 寄存器
impl<'a, T> Drop for LockGuard<'a, T> {
    fn drop(&mut self) {
        {
            self.guard.take()
        };
        unsafe { llvm_asm!("csrs sstatus, $0" :: "r"(self.sstatus & 2) :: "volatile") };
    }
}

impl<'a, T> core::ops::Deref for LockGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.guard.as_ref().unwrap().deref()
    }
}

impl<'a, T> core::ops::DerefMut for LockGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.guard.as_mut().unwrap().deref_mut()
    }
}
