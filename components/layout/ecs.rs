/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use evmap;
use hash_hasher::HashBuildHasher;
use hashbrown::HashMap;
use parking_lot::RwLock;
use std::sync::atomic::{AtomicUsize, Ordering};

pub type Component<T> = RwLock<HashMap<Entity, RwLock<T>, HashBuildHasher>>;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Entity(pub u32);

pub struct EntityGenerator {
    state: AtomicUsize,
}

impl EntityGenerator {
    pub fn next(&mut self) -> Entity {
        const A: usize = 6364136223846793005;
        const SHIFT: usize = 33;
        loop {
            let old = self.state.load(Ordering::Relaxed);
            let new = old * A + 1;
            if self.state.compare_and_swap(old, new, Ordering::SeqCst) == old {
                return Entity((new >> SHIFT) as u32)
            }
        }
    }
}
