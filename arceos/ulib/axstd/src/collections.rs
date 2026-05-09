use core::hash::{BuildHasher, Hasher};
use core::ops::{Deref, DerefMut};

use arceos_api::modules::axhal::misc;

pub struct SimpleHasher(u64);

impl Hasher for SimpleHasher {
    fn finish(&self) -> u64 { self.0 }

    fn write(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.0 ^= b as u64;
            self.0 = self.0.wrapping_mul(0x01000193);
        }
    }
    
}

pub struct RandomState {
    seed: u64,
}

impl RandomState {
    fn new() -> Self {
        let r = misc::random() as u64;
        Self { seed: r }
    }
}

impl BuildHasher for RandomState {
    type Hasher = SimpleHasher;
    fn build_hasher(&self) -> Self::Hasher {
        SimpleHasher(self.seed)
    }
    
}

impl Default for RandomState {
    fn default() -> Self {
        Self::new()
    }
}

pub struct HashMap<K, V, S = RandomState> {
    base: hashbrown::HashMap<K, V, S>,
}

impl <K, V> HashMap<K, V, RandomState> {
    pub fn new() -> Self {
        Self { base: hashbrown::HashMap::with_hasher(RandomState::new()) }
    }
}

impl<K, V, S> Deref for HashMap<K, V, S> {
    type Target = hashbrown::HashMap<K, V, S>;
    fn deref(&self) -> &Self::Target { &self.base }
}
impl<K, V, S> DerefMut for HashMap<K, V, S> {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.base }
}
