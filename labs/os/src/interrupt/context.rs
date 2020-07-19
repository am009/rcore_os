use core::mem::zeroed;
use riscv::register::sstatus::{self, Sstatus, SPP::*};

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Context {
    pub x: [usize; 32],
    pub sstatus: Sstatus,
    pub sepc: usize,
}

impl Default for Context {
    fn default() -> Self {
        unsafe { zeroed() }
    }
}

impl Context {
    pub fn sp(&self) -> usize {
        self.x[2]
    }
    pub fn set_sp(&mut self, value: usize) -> &mut Self {
        self.x[2] = value;
        self
    }
    pub fn ra(&self) -> usize {
        self.x[1]
    }
    pub fn set_ra(&mut self, value: usize) -> &mut Self {
        self.x[1] = value;
        self
    }
    pub fn set_arguments(&mut self, arguments: &[usize]) -> &mut Self {
        assert!(arguments.len() <= 8);
        self.x[10..(10 + arguments.len())].copy_from_slice(arguments);
        self
    }
    pub fn new(
        stack_top: usize,
        entry_point: usize,
        arguments: Option<&[usize]>,
        is_user: bool,
    ) -> Self {
        let mut context = Self::default();
        context.set_sp(stack_top).set_ra(-1isize as usize);
        if let Some(args) = arguments {
            context.set_arguments(args);
        }
        context.sepc = entry_point;
        context.sstatus = sstatus::read();
        if is_user {
            context.sstatus.set_spp(User);
        } else {
            context.sstatus.set_spp(Supervisor)
        }

        // 这样设置 SPIE 位，使得替换 sstatus 后关闭中断，
        // 而在 sret 到用户线程时开启中断。详见 SPIE 和 SIE 的定义
        context.sstatus.set_spie(true);

        context
    }
}
