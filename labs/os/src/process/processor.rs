
use super::*;
use hashbrown::HashSet;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref PROCESSOR: Lock<Processor> = Lock::new(Default::default());
}

#[derive(Default)]
pub struct Processor {
    current_thread: Option<Arc<Thread>>,
    scheduler: SchedulerImpl<Arc<Thread>>,
    sleeping_threads: HashSet<Arc<Thread>>
}

impl Processor {
    pub fn current_thread(&self) -> Arc<Thread> {
        self.current_thread.as_ref().unwrap().clone()
    }

    pub fn run(&mut self) -> ! {
        extern "C" {
            fn __restore(context: usize);
        }
        if self.current_thread.is_none() {
            panic!("no thread to run, shutting down!!");
        }
        let context = self.current_thread().prepare();
        unsafe {
            __restore(context as usize);
        }
        unreachable!()
    }
    pub fn prepare_next_thread(&mut self) -> *mut Context {
        loop {
            if let Some(next_thread) = self.scheduler.get_next() {
                let context = next_thread.prepare();
                self.current_thread = Some(next_thread);
                return context
            } else {
                // 没有活跃线程
                if self.sleeping_threads.is_empty() {
                    panic!("no thread to schedule or sleeping!! shutting down...");
                } else {
                    crate::interrupt::wait_for_interrupt();
                }
            }
        }
    }
    pub fn add_thread(&mut self, thread: Arc<Thread>) {
        if self.current_thread.is_none() {
            self.current_thread = Some(thread.clone());
        }
        self.scheduler.add_thread(thread, 0);
    }
    pub fn wake_thread(&mut self, thread: Arc<Thread>) {
        thread.inner().sleeping = false;
        self.sleeping_threads.remove(&thread);
        self.scheduler.add_thread(thread, 0);
    }
    pub fn park_current_thread(&mut self, context: &Context) {
        self.current_thread().park(*context); // ????这里会夺走所有权吗?...
    }
    pub fn sleep_current_thread(&mut self) {
        let current_thread = self.current_thread();
        current_thread.inner().sleeping = true;
        self.scheduler.remove_thread(&current_thread);
        self.sleeping_threads.insert(current_thread);
    }
    pub fn kill_current_thread(&mut self) {
        let thread = self.current_thread.take().unwrap();
        self.scheduler.remove_thread(&thread);
    }
}