use anyhow::{anyhow, Result};
use std::ops::{Add, AddAssign, Deref, Mul};
// use std::ops::{Index, Deref};
pub struct Vector<T> {
    data: Vec<T>,
}

// pretend this is a heavy computation, CPU intensive, so we want to move it to a thread. // 假装这是一个计算量重的任务，CPU 密集型，所以我们想把它移到一个线程中。
pub fn dot_product<T>(a: Vector<T>, b: Vector<T>) -> Result<T>
where
    T: Mul<Output = T> + Add<Output = T> + AddAssign + Default + Copy,
{
    if a.len() != b.len() {
        // a.len => a.data.len(), (通过 deref trait 实现的)
        return Err(anyhow!("Vector dimensions do not match"));
    }
    let mut sum = T::default();
    for i in 0..a.len() {
        sum += a[i] * b[i];
    }
    Ok(sum)
}

impl<T> Vector<T> {
    pub fn new(data: impl Into<Vec<T>>) -> Self {
        Self { data: data.into() }
    }

    // pub fn len(&self) -> usize {
    //     self.data.len()
    // }

    // pub fn iter(&self) -> std::slice::Iter<T> {
    //     self.data.iter()
    // }
}

// 方法一：实现 Deref trait
// 为 Vector<T> 实现 Deref trait，这样，我们就可以通过 *a 来访问 Vector<T> 中的 Vec<T>。
impl<T> Deref for Vector<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

// 方法二：实现 Index trait
// 为 Vector<T> 实现 Index trait，这样，我们就可以通过 a[i] 来访问 Vector<T> 中的元素。
// 这样，在 fn dot_product<T>(a: Vector<T>, b: Vector<T>) -> Result<T>，可以实现 a[i] * b[i] 的累加。
// impl<T> Index<usize> for Vector<T> {
//     type Output = T;

//     fn index(&self, index: usize) -> &Self::Output {
//         &self.data[index]
//     }
// }
