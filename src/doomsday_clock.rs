use std::sync::atomic::{AtomicUsize,Ordering};

#[derive(Debug)]
pub struct DoomsdayClock {
    counter: AtomicUsize,
}

impl DoomsdayClock {
    pub fn new(initial: usize) -> DoomsdayClock {
        DoomsdayClock {
            counter: AtomicUsize::new(initial)
        }
    }
    fn raw_tick(&self) -> (bool, usize) {
        const SET_ORDER: Ordering = Ordering::SeqCst;
        const FETCH_ORDER: Ordering = Ordering::SeqCst;
        // fetch_update lets us make this with less CAS logic load on our poor
        // brains, but is still an unstable feature
        let mut prev = self.counter.load(FETCH_ORDER);
        loop {
            if prev == 0 { return (false, 0) }
            let next = prev - 1;
            match self.counter.compare_exchange_weak(prev, next, SET_ORDER, FETCH_ORDER) {
                Ok(_) => return (true, prev),
                Err(next_prev) => prev = next_prev
            }
        }
    }
    /// Returns true if we successfully claimed a tick.
    pub fn tick(&self) -> bool {
        self.raw_tick().0
    }
    pub fn tick_loudly(&self) -> bool {
        let ret = self.raw_tick();
        if ret.0 {
            eprint!("\r{}  \r", ret.1);
        }
        ret.0
    }
}
