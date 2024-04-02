use std::cell::RefCell;
use std::sync::atomic::{AtomicU64, Ordering};

pub type Sequence = u64;
pub type AtomicSequence = AtomicU64;

pub static SEQUENCE_GLOBAL: AtomicSequence = AtomicSequence::new(0);

thread_local! {
    pub static SEQUENCER: RefCell<Sequencer> = RefCell::new(Sequencer::new(Sequencer::DEFAULT_STEP, Sequencer::DEFAULT_LAG));
}

pub struct Sequencer {
    local: Sequence,
    target: Sequence,

    step: Sequence,
    lag: Sequence,
}

impl Sequencer {
    const DEFAULT_LAG: Sequence = Self::DEFAULT_STEP * 16;
    const DEFAULT_STEP: Sequence = 128;

    pub const fn new(step: Sequence, lag: Sequence) -> Self {
        Self {
            local: 0,
            target: 0,
            step,
            lag,
        }
    }

    pub fn global(&self) -> Sequence {
        SEQUENCE_GLOBAL.load(Ordering::Relaxed)
    }

    pub fn local(&self) -> Sequence {
        self.local
    }

    pub fn inc(&mut self) -> Sequence {
        self.try_alloc();
        let res = self.local;
        self.local += 1;
        res
    }

    #[inline(always)]
    fn try_alloc(&mut self) {
        if self.local == self.target
            || self.local + self.lag < SEQUENCE_GLOBAL.load(Ordering::Relaxed)
        {
            self.alloc()
        }
    }

    #[inline(always)]
    fn alloc(&mut self) {
        self.local = SEQUENCE_GLOBAL.fetch_add(self.step, Ordering::Relaxed);
        self.target = self.local + self.step;
    }
}
