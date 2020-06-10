use std::ops::Range;
use std::u128;

pub(crate) struct IDGenerator {
    inner: Range<u128>
}

impl IDGenerator {
    pub(crate) fn new() -> Self {
        Self { inner: 0..(u128::MAX) }
    }
    pub(crate) fn next(&mut self) -> u128 {
        self.inner.next().unwrap()
    }
}

#[macro_export]
macro_rules! map {
    ($($x:expr => $y:expr),*) => {{
        #[allow(unused_mut)]
        let mut temp_hash_map = std::collections::HashMap::new();
        $(
            temp_hash_map.insert($x, $y);
        )*
        temp_hash_map
    }}
}

#[macro_export]
macro_rules! set {
    ($($x:expr),*) => {{
        #[allow(unused_mut)]
        let mut temp_hash_set = std::collections::HashSet::new();
        $(
            temp_hash_set.insert($x);
        )*
        temp_hash_set
    }}
}

