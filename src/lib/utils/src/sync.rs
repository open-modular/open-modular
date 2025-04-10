use std::{
    sync::{
        Arc,
        Condvar,
        Mutex,
        MutexGuard,
        atomic::{
            AtomicU64,
            Ordering,
        },
    },
    thread,
};

use derive_more::with_trait::Debug;
use fancy_constructor::new;
use oneshot::{
    Receiver,
    Sender,
};

// =================================================================================================
// Sync
// =================================================================================================

// Barrier Group

#[derive(new, Debug)]
#[new(args(panic: BarrierGroupPanic))]
pub struct BarrierGroup(
    #[new(val = Arc::new(BarrierGroupState::new(panic)))] Arc<BarrierGroupState>,
);

impl BarrierGroup {
    #[must_use]
    pub fn barrier(&self) -> Barrier {
        self.0.data.lock().expect("lock to be obtained").active += 1;

        Barrier::new(Arc::clone(&self.0))
    }
}

impl Default for BarrierGroup {
    fn default() -> Self {
        Self::new(BarrierGroupPanic::default())
    }
}

// -------------------------------------------------------------------------------------------------

// Barrier

#[derive(new, Debug)]
pub struct Barrier(Arc<BarrierGroupState>);

impl Barrier {
    pub fn wait(&mut self) {
        let mut data = self.0.data.lock().expect("lock to be obtained");

        data.waiting += 1;

        let generation = data.generation;

        self.release(&mut data);

        let data = self
            .0
            .cvar
            .wait_while(data, |data| generation == data.generation)
            .expect("cvar wait to to return");

        if data.poisoned {
            drop(data);
            panic!("poisoned barrier");
        }
    }
}

impl Barrier {
    fn release(&self, data: &mut MutexGuard<'_, BarrierGroupData>) {
        if data.release() {
            self.0.cvar.notify_all();
        }
    }
}

impl Drop for Barrier {
    fn drop(&mut self) {
        let mut data = self.0.data.lock().expect("lock to be obtained");

        data.active -= 1;

        if self.0.panic == BarrierGroupPanic::Poison && thread::panicking() {
            data.poisoned = true;
        }

        self.release(&mut data);
    }
}

// -------------------------------------------------------------------------------------------------

// Panic

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum BarrierGroupPanic {
    Continue,
    Poison,
}

impl Default for BarrierGroupPanic {
    fn default() -> Self {
        Self::Continue
    }
}

// -------------------------------------------------------------------------------------------------

// State

#[derive(new, Debug)]
struct BarrierGroupState {
    #[new(default)]
    cvar: Condvar,
    #[new(default)]
    data: Mutex<BarrierGroupData>,
    panic: BarrierGroupPanic,
}

// -------------------------------------------------------------------------------------------------

// Data

#[derive(Debug, Default)]
struct BarrierGroupData {
    active: usize,
    waiting: usize,
    generation: usize,
    poisoned: bool,
}

impl BarrierGroupData {
    fn release(&mut self) -> bool {
        if self.waiting >= self.active || self.poisoned {
            self.generation = self.generation.wrapping_add(1);
            self.waiting = 0;

            true
        } else {
            false
        }
    }
}

// -------------------------------------------------------------------------------------------------

// Pending/Value

static CORRELATION: AtomicU64 = AtomicU64::new(0);

#[derive(new, Debug)]
pub struct Pending<T> {
    pub correlation: u64,
    #[debug(skip)]
    receiver: Receiver<T>,
}

impl<T> Pending<T> {
    #[must_use]
    pub fn create() -> (Value<T>, Self) {
        let channels = oneshot::channel();
        let correlation = CORRELATION.fetch_add(1, Ordering::Relaxed);

        (
            Value::new(correlation, channels.0),
            Self::new(correlation, channels.1),
        )
    }

    #[must_use]
    pub fn value(&self) -> Option<T> {
        self.receiver
            .has_message()
            .then(|| unsafe { self.receiver.recv_ref().unwrap_unchecked() })
    }
}

#[derive(new, Debug)]
pub struct Value<T> {
    pub correlation: u64,
    #[debug(skip)]
    sender: Sender<T>,
}

impl<T> Value<T> {
    pub fn set(self, value: T) {
        self.sender
            .send(value)
            .expect("value to be sent successfully");
    }
}
