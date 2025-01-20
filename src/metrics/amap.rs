use std::{
    collections::HashMap,
    fmt,
    sync::{
        atomic::{AtomicI64, Ordering},
        Arc,
    },
};

use anyhow::{anyhow, Result};

#[derive(Debug)]
pub struct AmapMetrics {
    data: Arc<HashMap<&'static str, AtomicI64>>, // 因为 Arc 实现了 send 和 sync，所以可以跨线程共享
}
// Suitable for scenarios where you have a fixed set of keys known at compile time and need to perform frequent concurrent updates to the values.
// Example: A metrics system where the keys are predefined metric names, and the values are counters that are incremented by multiple threads.

// The iterator yields references to the elements of the slice, so the type of the elements is &&'static str.
// The closure |&name| (name, AtomicI64::new(0)):
// &name destructures the reference, so name is a &'static str.
// The closure returns a tuple (name, AtomicI64::new(0)), where name is the key and AtomicI64::new(0) is the value.
// The map method transforms each element of the iterator into a tuple (name, AtomicI64::new(0)).
// The collect method consumes the iterator and collects the elements into a collection.
// In this case, the elements are collected into a HashMap<&'static str, AtomicI64>.
// The type of the collection is inferred based on the context, which is a HashMap in this case.
impl AmapMetrics {
    pub fn new(metric_names: &[&'static str]) -> Self {
        let map = metric_names
            .iter()
            .map(|&name| (name, AtomicI64::new(0)))
            .collect();
        AmapMetrics {
            data: Arc::new(map),
        }
    }

    // AsRef is a trait in Rust's standard library that provides a way to convert a value to a reference of another type.
    // It is commonly used to allow functions to accept arguments of multiple types that can be converted to a reference of a specific type.
    // in this case, key: impl AsRef<str> means that the key parameter can be of any type that can be converted to a reference to a string,
    // which is &str.
    // how many other types can be converted to &str?
    // The AsRef trait is implemented for many types in Rust, including String, &str, Path, and OsStr.
    pub fn inc(&self, key: impl AsRef<str>) -> Result<()> {
        let key = key.as_ref();
        let counter = self
            .data
            .get(key)
            .ok_or_else(|| anyhow!("key {} not found", key))?;
        counter.fetch_add(1, Ordering::Relaxed); // fetch_add 是读，load 是写
        Ok(())
    }
}

impl Clone for AmapMetrics {
    fn clone(&self) -> Self {
        AmapMetrics {
            data: Arc::clone(&self.data),
        }
    }
}

impl fmt::Display for AmapMetrics {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (key, value) in self.data.iter() {
            writeln!(f, "{}: {}", key, value.load(Ordering::Relaxed))?; // fetch_add 是读，load 是写
        }
        Ok(())
    }
}
