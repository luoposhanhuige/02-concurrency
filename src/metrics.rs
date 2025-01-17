// metrics data structure
// functionality: inc/dec/snapshot
use anyhow::{anyhow, Result};
use std::{
    collections::HashMap,
    fmt,
    sync::{Arc, RwLock}, // 用 RwLock 替换 Mutex，后者不区分 read 和 write，前者区分 read 和 write
};

// Arc (Atomic Reference Counting):

// Arc is a thread-safe reference-counting pointer. It enables multiple ownership of the same data by keeping track of the number of references to the data.
// When the last Arc pointer to the data is dropped, the data is deallocated.
// Arc is used to share data across threads safely.
// Mutex (Mutual Exclusion):

// Mutex is a synchronization primitive that provides mutual exclusion, allowing only one thread to access the data at a time.
// It ensures that only one thread can lock the Mutex and access the data, preventing data races.
// When a thread locks the Mutex, other threads attempting to lock it will block until the Mutex is unlocked.
// HashMap:

// HashMap is a collection that stores key-value pairs. In this context, it is used to store metrics, where the key is a String (e.g., metric name) and the value is an i64 (e.g., metric count).

// Arc<Mutex<...>> is a powerful pattern for safely sharing and modifying data across multiple threads.
// Arc provides shared ownership and thread-safe reference counting.
// Mutex ensures mutual exclusion, preventing data races.
// This pattern is commonly used in Rust for concurrent programming scenarios where shared mutable state is required.

// Arc<Mutex<...>> is a common pattern in Rust for sharing data across threads.

// metrics.clone()
// Arc (Atomic Reference Counting):

// Arc is a thread-safe reference-counting pointer that allows multiple ownership of the same data.
// When you clone an Arc, it increments the reference count, but the underlying data is not copied. Instead, all clones share the same data.
// Mutex (Mutual Exclusion):

// Mutex is a synchronization primitive that ensures only one thread can access the data at a time.
// This prevents data races and ensures safe concurrent access to the shared data.
#[derive(Debug, Clone)]
pub struct Metrics {
    data: Arc<RwLock<HashMap<String, i64>>>,
}

impl Metrics {
    pub fn new() -> Metrics {
        Metrics {
            data: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    // data.entry
    // data is a HashMap<String, i64>. // literally, data is a Mutex<HashMap<String, i64>> which implements Deref trait.
    // data.entry(key) accesses the entry for the given key in the HashMap.
    // The entry method returns an Entry enum, which can be either Occupied or Vacant.
    // or_insert(value) is called on the Entry.
    // If the entry is Vacant, or_insert inserts the provided value and returns a mutable reference to it.
    // If the entry is Occupied, or_insert returns a mutable reference to the existing value.

    // inc, 顾名思义，就是增加某个 key 对应的计数器的值
    pub fn inc(&self, key: impl Into<String>) -> Result<()> {
        // let mut data = self.data.lock().map_err(|e| anyhow!(e.to_string()))?; // MutexGuard<HashMap<String, i64>>
        let mut data = self.data.write().map_err(|e| anyhow!(e.to_string()))?; // RwLock 区分 read 和 write
        let count = data.entry(key.into()).or_insert(0); // returns a mutable reference to the value
        *count += 1;
        Ok(())
    }

    // pub fn dec(&self, key: impl Into<String>) -> Result<()>  {
    //     let mut data = self.data.lock().map_err(|e| anyhow!(e.to_string()))?;
    //     let count = data.entry(key.into()).or_insert(0);
    //     *count -= 1;
    //     Ok(())
    // }

    // 1
    // The map_err method is used to transform the error type, not to propagate it.
    // The propagation of the error is handled by the ? operator.
    // 2
    // The ? operator is used to propagate errors.
    // When used on a Result, it will:
    // Return the Ok variant's value if the Result is Ok.
    // Return early from the function with the Err variant's value if the Result is Err.
    // MutexGuard:
    // 3
    // When self.data.lock() is called, it returns a Result<MutexGuard<HashMap<String, i64>>, PoisonError<MutexGuard<HashMap<String, i64>>>>.
    // If the Result is Ok, the ? operator unwraps it to get the MutexGuard<HashMap<String, i64>>.
    // clone Method:
    // 4
    // The clone method is called on the MutexGuard<HashMap<String, i64>>.
    // The MutexGuard implements Deref, so calling clone on it effectively calls clone on the underlying HashMap<String, i64>.

    // 把整个 Metrics 的数据结构 clone 一份，返回给调用者，
    // 这个跟 metircs.clone() 是不一样的，metrics.clone() 是返回一个 Metrics 的副本，而这个是返回 Metrics 内部的数据结构的副本
    pub fn snapshot(&self) -> Result<HashMap<String, i64>> {
        Ok(self
            .data
            .read()
            .map_err(|e| anyhow!(e.to_string()))?
            .clone())
    }
}

impl Default for Metrics {
    fn default() -> Self {
        Self::new()
    }
}
// 与 metrics.snapshot 不同，前者用到 .clone()，后者没有用到
impl fmt::Display for Metrics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let data = self.data.read().map_err(|_e| fmt::Error)?;
        for (key, value) in data.iter() {
            writeln!(f, "{}: {}", key, value)?;
        }
        Ok(())
    }
}
