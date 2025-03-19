use std::thread::Builder;

use thread_priority::ThreadBuilder;

// =================================================================================================
// Thread
// =================================================================================================

pub trait BuilderExt {
    #[must_use]
    fn named(self, name: impl Into<String>) -> Self;
}

impl BuilderExt for Builder {
    fn named(self, name: impl Into<String>) -> Self {
        self.name(name.into())
    }
}

impl BuilderExt for ThreadBuilder {
    fn named(self, name: impl Into<String>) -> Self {
        self.name(name.into())
    }
}
