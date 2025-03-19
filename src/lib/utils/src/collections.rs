use std::collections::VecDeque;

use derive_more::with_trait::Debug;
use tracing::{
    error,
    trace,
};

// =================================================================================================
// Collections
// =================================================================================================

// IndexMap

#[derive(Debug)]
pub struct IndexMap<const C: usize, T> {
    empty: VecDeque<usize>,
    index: usize,
    #[debug("[{{ .. }}; {}]", C)]
    storage: Box<[Option<T>; C]>,
}

impl<const C: usize, T> IndexMap<C, T> {
    pub fn add(&mut self, item: T) -> usize
    where
        T: Debug,
    {
        trace!(?item, "adding item to index map");

        if let Some(index) = self.empty.pop_front() {
            trace!(index, "reusing previously used entry");

            unsafe {
                *self.storage.get_unchecked_mut(index) = Some(item);
            }

            index
        } else if self.index < C {
            let index = self.index;

            self.index += 1;

            trace!(index, "using previously unused entry");

            unsafe {
                *self.storage.get_unchecked_mut(index) = Some(item);
            }

            index
        } else {
            error!(C, "error (storage capacity exceeded)");
            panic!("storage capacity ({C}) exceeded");
        }
    }

    pub fn entries(&self) -> impl Iterator<Item = (usize, &T)> {
        self.storage
            .iter()
            .enumerate()
            .take(self.index)
            .filter_map(|(i, t)| t.as_ref().map(|t| (i, t)))
    }

    pub fn entries_mut(&mut self) -> impl Iterator<Item = (usize, &mut T)> {
        self.storage
            .iter_mut()
            .enumerate()
            .take(self.index)
            .filter_map(|(i, t)| t.as_mut().map(|t| (i, t)))
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.index == self.empty.len()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.index - self.empty.len()
    }

    pub fn retain(&mut self, predicate: impl Fn(&T) -> bool) {
        self.storage.iter_mut().enumerate().for_each(|(i, t)| {
            if t.as_ref().is_some_and(|t| !predicate(t)) {
                *t = None;
                self.empty.push_back(i);
            }
        });
    }

    pub fn values(&self) -> impl Iterator<Item = &T> {
        self.storage.iter().take(self.index).flatten()
    }

    pub fn values_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.storage.iter_mut().take(self.index).flatten()
    }
}

impl<const C: usize, T> IndexMap<C, T> {
    #[must_use]
    pub unsafe fn get_unchecked(&self, index: usize) -> &T {
        unsafe {
            self.storage
                .get_unchecked(index)
                .as_ref()
                .unwrap_unchecked()
        }
    }

    pub unsafe fn remove_unchecked(&mut self, index: usize) {
        let item = unsafe { self.storage.get_unchecked_mut(index) };

        if item.is_some() {
            *item = None;
            self.empty.push_back(index);
        }
    }
}

impl<const C: usize, T> Default for IndexMap<C, T> {
    fn default() -> Self {
        Self {
            empty: VecDeque::with_capacity(C),
            index: 0,
            storage: Box::new([const { None }; C]),
        }
    }
}
