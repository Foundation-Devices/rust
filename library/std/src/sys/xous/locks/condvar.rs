use super::mutex::Mutex;
use crate::sync::atomic::{AtomicUsize, Ordering::SeqCst};
use crate::sys::services::ticktimer;
use crate::sys_common::lazy_box::LazyInit;
use crate::time::Duration;

static CONDVAR_INDEX: AtomicUsize = AtomicUsize::new(1);

// The implementation is inspired by Andrew D. Birrell's paper
// "Implementing Condition Variables with Semaphores"

pub struct Condvar {
    counter: AtomicUsize,
    index: AtomicUsize,
}

impl LazyInit for Condvar {
    fn init() -> Box<Self> {
        let this = Self::new();
        this.index.store(CONDVAR_INDEX.fetch_add(1, SeqCst), SeqCst);
        Box::new(this)
    }
}

unsafe impl Send for Condvar {}
unsafe impl Sync for Condvar {}

impl Condvar {
    pub const fn new() -> Condvar {
        Condvar { counter: AtomicUsize::new(0), index: AtomicUsize::new(0) }
    }

    pub fn notify_one(&self) {
        if self.counter.load(SeqCst) > 0 {
            self.counter.fetch_sub(1, SeqCst);
            xous::send_message(
                ticktimer(),
                xous::Message::new_scalar(
                    9, /* NotifyCondition */
                    self.index.load(SeqCst),
                    1,
                    0,
                    0,
                ),
            )
            .expect("Ticktimer: failure to send NotifyCondition command");
        }
    }

    pub fn notify_all(&self) {
        let counter = self.counter.swap(0, SeqCst);
        xous::send_message(
            ticktimer(),
            xous::Message::new_scalar(
                9, /* NotifyCondition */
                self.index.load(SeqCst),
                counter,
                0,
                0,
            ),
        )
        .expect("Ticktimer: failure to send NotifyCondition command");
    }

    pub unsafe fn wait(&self, mutex: &Mutex) {
        self.counter.fetch_add(1, SeqCst);
        unsafe { mutex.unlock() };
        xous::send_message(
            ticktimer(),
            xous::Message::new_blocking_scalar(
                8, /* WaitForCondition */
                self.index.load(SeqCst),
                0,
                0,
                0,
            ),
        )
        .expect("Ticktimer: failure to send WaitForCondition command");
        unsafe { mutex.lock() };
    }

    pub unsafe fn wait_timeout(&self, mutex: &Mutex, dur: Duration) -> bool {
        self.counter.fetch_add(1, SeqCst);
        unsafe { mutex.unlock() };
        let millis = dur.as_millis() as usize;
        let result = xous::send_message(
            ticktimer(),
            xous::Message::new_blocking_scalar(
                8, /* WaitForCondition */
                self.index.load(SeqCst),
                millis,
                0,
                0,
            ),
        )
        .expect("Ticktimer: failure to send WaitForCondition command");
        unsafe { mutex.lock() };

        xous::Result::Scalar1(0) == result
    }
}
