// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::{
    fmt::{self, Debug, Formatter},
    iter::Enumerate,
    marker::PhantomData,
    option::Option,
    slice,
};

pub struct IndexMap<K, V> {
    _keys: PhantomData<K>,
    values: Vec<Option<V>>,
}

impl<K, V> IndexMap<K, V> {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    // `Iter` does implement `Iterator`, but it has an additional bound on `K`.
    #[allow(clippy::iter_not_returning_iterator)]
    #[must_use]
    pub fn iter(&self) -> Iter<K, V> {
        Iter {
            _keys: PhantomData,
            base: self.values.iter().enumerate(),
        }
    }
}

impl<K: Into<usize>, V> IndexMap<K, V> {
    pub fn insert(&mut self, key: K, value: V) {
        let index = key.into();
        if index >= self.values.len() {
            self.values.resize_with(index + 1, || None);
        }
        self.values[index] = Some(value);
    }

    pub fn get(&self, key: K) -> Option<&V> {
        let index: usize = key.into();
        self.values.get(index).and_then(Option::as_ref)
    }

    pub fn get_mut(&mut self, key: K) -> Option<&mut V> {
        let index: usize = key.into();
        self.values.get_mut(index).and_then(Option::as_mut)
    }
}

impl<K, V: Debug> Debug for IndexMap<K, V> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("IndexMap")
            .field("values", &self.values)
            .finish()
    }
}

impl<K, V> Default for IndexMap<K, V> {
    fn default() -> Self {
        Self {
            _keys: PhantomData,
            values: Vec::default(),
        }
    }
}

impl<'a, K: From<usize>, V> IntoIterator for &'a IndexMap<K, V> {
    type Item = (K, &'a V);

    type IntoIter = Iter<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

pub struct Iter<'a, K, V> {
    _keys: PhantomData<K>,
    base: Enumerate<slice::Iter<'a, Option<V>>>,
}

impl<'a, K: From<usize>, V> Iterator for Iter<'a, K, V> {
    type Item = (K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let (index, Some(value)) = self.base.next()? {
                break Some((index.into(), value));
            }
        }
    }
}
