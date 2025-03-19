use std::sync::{
    Arc,
    atomic::{
        AtomicBool,
        Ordering,
    },
};

// =================================================================================================
// Control
// =================================================================================================

// Exit

#[derive(Clone, Debug, Default)]
pub struct Exit {
    exit: Arc<AtomicBool>,
}

impl Exit {
    #[must_use]
    pub fn triggered(&self) -> bool {
        self.exit.load(Ordering::Relaxed)
    }

    pub fn trigger(&mut self) {
        self.exit.store(true, Ordering::Relaxed);
    }
}
