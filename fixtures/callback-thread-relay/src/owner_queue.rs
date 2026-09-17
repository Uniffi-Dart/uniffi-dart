//! Experimental single-isolate queue shared by synchronous calls and listener wakes.
use std::{
    collections::VecDeque,
    sync::{Condvar, Mutex, OnceLock},
};

pub type Job = Box<dyn FnOnce() + Send>;
type Wake = extern "C" fn();
static WAKE: OnceLock<Wake> = OnceLock::new();

#[derive(Default)]
struct State {
    jobs: VecDeque<Job>,
    active: usize,
    notified: bool,
    closed: bool,
}

#[derive(Default)]
pub struct Queue {
    state: Mutex<State>,
    ready: Condvar,
}

impl Queue {
    // The listener only posts a message; it cannot synchronously reenter Dart.
    // Serialize notification with shutdown so close cannot race a native wake.
    fn notify(state: &mut State) {
        if state.active == 0 && !state.jobs.is_empty() && !state.notified {
            state.notified = true;
            WAKE.get().expect("listener must be installed first")();
        }
    }
    pub fn push(&self, job: Job) {
        let mut state = self.state.lock().unwrap();
        assert!(!state.closed, "dispatch after queue closed");
        state.jobs.push_back(job);
        Self::notify(&mut state);
        self.ready.notify_one();
    }
    pub fn enter(&self, from_listener: bool) {
        let mut state = self.state.lock().unwrap();
        assert!(!state.closed, "enter after queue closed");
        if from_listener {
            assert!(state.notified, "unexpected listener notification");
            state.notified = false;
        }
        state.active += 1;
    }
    pub fn leave(&self) {
        let mut state = self.state.lock().unwrap();
        state.active -= 1;
        Self::notify(&mut state);
    }
    pub fn pump_one(&self, wait: bool) -> bool {
        let job = {
            let mut state = self.state.lock().unwrap();
            loop {
                if let Some(job) = state.jobs.pop_front() {
                    break Some(job);
                }
                if !wait {
                    break None;
                }
                state = self.ready.wait(state).unwrap();
            }
        };
        if let Some(job) = job {
            job();
            true
        } else {
            false
        }
    }
    pub fn close(&self) -> bool {
        let mut state = self.state.lock().unwrap();
        if state.active != 0 || state.notified || !state.jobs.is_empty() {
            return false;
        }
        state.closed = true;
        true
    }
    pub fn stat(&self, which: u32) -> u64 {
        let state = self.state.lock().unwrap();
        match which {
            6 => state.jobs.len() as u64,
            7 => state.notified as u64,
            8 => state.closed as u64,
            _ => u64::MAX,
        }
    }
}

#[no_mangle]
pub extern "C" fn relay_set_listener(wake: Wake) {
    assert!(WAKE.set(wake).is_ok(), "one listener per process in this prototype");
}
