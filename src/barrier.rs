use std::sync::atomic::{AtomicI64, Ordering};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct GlobalSourceBus {
    value: AtomicI64,
}

#[wasm_bindgen]
impl GlobalSourceBus {
    #[wasm_bindgen(constructor)]
    pub fn new(initial: i64) -> Self {
        Self {
            value: AtomicI64::new(initial),
        }
    }

    pub fn get(&self) -> i64 {
        self.value.load(Ordering::Relaxed)
    }

    pub fn set(&self, val: i64) {
        self.value.store(val, Ordering::Relaxed);
    }

    pub fn add(&self, delta: i64) -> i64 {
        self.value.fetch_add(delta, Ordering::Relaxed)
    }

    pub fn reset(&self) {
        self.value.store(0, Ordering::Relaxed);
    }
}

impl Default for GlobalSourceBus {
    fn default() -> Self {
        Self::new(0)
    }
}

#[wasm_bindgen]
pub struct GlobalSinkBus {
    value: AtomicI64,
}

#[wasm_bindgen]
impl GlobalSinkBus {
    #[wasm_bindgen(constructor)]
    pub fn new(initial: i64) -> Self {
        Self {
            value: AtomicI64::new(initial),
        }
    }

    pub fn get(&self) -> i64 {
        self.value.load(Ordering::Relaxed)
    }

    pub fn set(&self, val: i64) {
        self.value.store(val, Ordering::Relaxed);
    }

    pub fn add(&self, delta: i64) -> i64 {
        self.value.fetch_add(delta, Ordering::Relaxed)
    }

    pub fn reset(&self) {
        self.value.store(0, Ordering::Relaxed);
    }
}

impl Default for GlobalSinkBus {
    fn default() -> Self {
        Self::new(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_global_source_bus() {
        let bus = GlobalSourceBus::new(0);
        assert_eq!(bus.get(), 0);
        bus.add(100);
        assert_eq!(bus.get(), 100);
        bus.set(500);
        assert_eq!(bus.get(), 500);
        bus.reset();
        assert_eq!(bus.get(), 0);
    }

    #[test]
    fn test_global_sink_bus() {
        let bus = GlobalSinkBus::new(0);
        assert_eq!(bus.get(), 0);
        bus.add(50);
        assert_eq!(bus.get(), 50);
        bus.set(200);
        assert_eq!(bus.get(), 200);
        bus.reset();
        assert_eq!(bus.get(), 0);
    }
}
