/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! A persistent, thread-safe singly-linked list.

use std::mem;
use sync::Arc;

pub struct PersistentList<T> where T: Send + Sync {
    head: Option<Arc<PersistentListEntry<T>>>,
    length: uint,
}

struct PersistentListEntry<T> where T: Send + Sync {
    value: T,
    next: Option<Arc<PersistentListEntry<T>>>,
}

impl<T> PersistentList<T> where T: Send + Sync {
    #[inline]
    pub fn new() -> PersistentList<T> {
        PersistentList {
            head: None,
            length: 0,
        }
    }

    #[inline]
    pub fn len(&self) -> uint {
        self.length
    }

    #[inline]
    pub fn first(&self) -> Option<&T> {
        match self.head {
            None => None,
            Some(ref head) => Some(&head.value),
        }
    }

    #[inline]
    pub fn prepend_elem(&self, value: T) -> PersistentList<T> {
        PersistentList {
            head: Some(Arc::new(PersistentListEntry {
                value: value,
                next: self.head.clone(),
            })),
            length: self.length + 1,
        }
    }

    #[inline]
    pub fn iter(&self) -> PersistentListIterator<T> {
        // This could clone (and would not need the lifetime if it did), but then it would incur
        // atomic operations on every call to `.next()`. Bad.
        PersistentListIterator {
            entry: match self.head {
                None => None,
                Some(ref head) => Some(&**head),
            },
        }
    }
}

impl<T> Clone for PersistentList<T> where T: Send + Sync {
    fn clone(&self) -> PersistentList<T> {
        PersistentList {
            head: self.head.clone(),
            length: self.length,
        }
    }
}

pub struct PersistentListIterator<'a,T> where T: 'a + Send + Sync {
    entry: Option<&'a PersistentListEntry<T>>,
}

impl<'a,T> Iterator<&'a T> for PersistentListIterator<'a,T> where T: Send + Sync {
    #[inline]
    fn next(&mut self) -> Option<&'a T> {
        let entry = match self.entry {
            None => return None,
            Some(entry) => {
                unsafe {
                    mem::transmute::<&'a PersistentListEntry<T>,
                                     &'static PersistentListEntry<T>>(entry)
                }
            }
        };
        let value = &entry.value;
        self.entry = match entry.next {
            None => None,
            Some(ref entry) => Some(&**entry),
        };
        Some(value)
    }
}

