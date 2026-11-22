use std::fmt::{Debug, Write};
use std::collections::HashMap;

pub fn fingerprint(tag: &str, input: &impl Debug) -> u64 {
    let mut sink = Fnv(0xCBF2_9CE4_8422_2325);
    let _ = write!(sink, "hd2|{tag}|{input:?}");
    sink.0
}

struct Fnv(u64);

impl Write for Fnv {
    fn write_str(&mut self, text: &str) -> std::fmt::Result {
        for byte in text.as_bytes() {
            self.0 ^= u64::from(*byte);
            self.0 = self.0.wrapping_mul(0x100_0000_01B3);
        }
        Ok(())
    }
}
use std::sync::{Mutex, MutexGuard, OnceLock};

use aurorix_render::memory::resource_cache_limit;

pub struct Memo<T> {
    entries: HashMap<String, Slot<T>>,
    used: usize,
    clock: u64,
    budget: usize,
}

struct Slot<T> {
    value: T,
    bytes: usize,
    seen: u64,
}

pub fn budget() -> usize {
    (resource_cache_limit() / 2).max(32 * 1024 * 1024)
}

impl<T> Memo<T> {
    pub fn new(budget: usize) -> Self {
        Self {
            entries: HashMap::new(),
            used: 0,
            clock: 0,
            budget: budget.max(1),
        }
    }

    pub fn get(&mut self, key: &str) -> Option<&T> {
        self.clock += 1;
        let clock = self.clock;
        let slot = self.entries.get_mut(key)?;
        slot.seen = clock;
        Some(&slot.value)
    }

    pub fn put(&mut self, key: String, value: T, bytes: usize) {
        if bytes > self.budget {
            return;
        }
        self.clock += 1;
        if let Some(previous) = self.entries.remove(&key) {
            self.used -= previous.bytes;
        }
        while self.used + bytes > self.budget {
            let Some(victim) = self
                .entries
                .iter()
                .min_by_key(|(_, slot)| slot.seen)
                .map(|(key, _)| key.clone())
            else {
                break;
            };
            if let Some(removed) = self.entries.remove(&victim) {
                self.used -= removed.bytes;
            }
        }
        self.used += bytes;
        self.entries.insert(
            key,
            Slot {
                value,
                bytes,
                seen: self.clock,
            },
        );
    }
}

pub fn lock<T: 'static>(cell: &'static OnceLock<Mutex<Memo<T>>>) -> MutexGuard<'static, Memo<T>> {
    cell.get_or_init(|| Mutex::new(Memo::new(budget())))
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

pub fn cached<T: Clone + 'static>(
    cell: &'static OnceLock<Mutex<Memo<T>>>,
    key: &str,
    build: impl FnOnce() -> Option<(T, usize)>,
) -> Option<T> {
    if let Some(hit) = lock(cell).get(key).cloned() {
        return Some(hit);
    }
    let (value, bytes) = build()?;
    lock(cell).put(key.to_string(), value.clone(), bytes);
    Some(value)
}
