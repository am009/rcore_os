use riscv::register::sstatus::Sstatus;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Context {
    pub x: [usize; 32],
    pub sstatus: Sstatus,
    pub sepc: usize
}