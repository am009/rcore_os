
use alloc::collections::LinkedList;

pub type SchedulerImpl<T> = HrrnScheduler<T>;

pub trait Scheduler<ThreadType: Clone + Eq>: Default {
    fn add_thread<T>(&mut self, thread: ThreadType, priority: T);
    fn get_next(&mut self) -> Option<ThreadType>;
    fn remove_thread(&mut self, thread: &ThreadType);
    fn set_priority<T>(&mut self, thread: ThreadType, priority: T);
}

struct HrrnThread<ThreadType: Clone + Eq> {
    birth_time: usize,
    service_count: usize,
    pub thread: ThreadType
}

pub struct HrrnScheduler<ThreadType: Clone + Eq> {
    current_time: usize,
    pool: LinkedList<HrrnThread<ThreadType>>
}

impl<ThreadType: Clone + Eq> Default for HrrnScheduler<ThreadType> {
    fn default() -> Self {
        Self {
            current_time: 0,
            pool: LinkedList::new()
        }
    }
}

impl<ThreadType: Clone + Eq> Scheduler<ThreadType> for HrrnScheduler<ThreadType> {
    fn add_thread<T>(&mut self, thread: ThreadType, _priority: T) {
        self.pool.push_back(HrrnThread {
            birth_time: self.current_time,
            service_count: 0,
            thread
        })
    }
    fn remove_thread(&mut self, thread: &ThreadType) {
        let mut removed = self.pool.drain_filter(|t| t.thread == *thread);
        // 确保只删除一个
        assert!(removed.next().is_some() && removed.next().is_none())
    }
    // 暂不支持优先级
    fn set_priority<T>(&mut self, _thread: ThreadType, _priority: T) {}
    fn get_next(&mut self) -> Option<ThreadType> {
        self.current_time += 1;
        let current_time = self.current_time; // borrow-check ??? 什么意思?
        if let Some(best) = self.pool.iter_mut().max_by(|x, y| {
            ((current_time - x.birth_time) * y.service_count)
                .cmp(&((current_time - y.birth_time) * x.service_count))
        }) {
            best.service_count += 1;
            Some(best.thread.clone())
        } else {
            None
        }
    }
}
