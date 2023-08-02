// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::{
    fmt::{self, Debug, Formatter},
    iter::Enumerate,
    marker::PhantomData,
    option::Option,
    slice, vec,
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

    // `Iter` does implement `Iterator`, but it has an additional bound on `K`.
    #[allow(clippy::iter_not_returning_iterator)]
    pub fn iter_mut(&mut self) -> IterMut<K, V> {
        IterMut {
            _keys: PhantomData,
            base: self.values.iter_mut().enumerate(),
        }
    }

    pub fn drain(&mut self) -> Drain<K, V> {
        Drain {
            _keys: PhantomData,
            base: self.values.drain(..).enumerate(),
        }
    }

    #[must_use]
    pub fn values(&self) -> Values<V> {
        Values {
            base: self.values.iter(),
        }
    }

    pub fn values_mut(&mut self) -> ValuesMut<V> {
        ValuesMut {
            base: self.values.iter_mut(),
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

    pub fn contains_key(&self, key: K) -> bool {
        let index: usize = key.into();
        self.values.get(index).map_or(false, Option::is_some)
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

impl<K, V: Clone> Clone for IndexMap<K, V> {
    fn clone(&self) -> Self {
        Self {
            _keys: PhantomData,
            values: self.values.clone(),
        }
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

impl<K: From<usize>, V> IntoIterator for IndexMap<K, V> {
    type Item = (K, V);

    type IntoIter = IntoIter<K, V>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            _keys: PhantomData,
            base: self.values.into_iter().enumerate(),
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

impl<K: Into<usize>, V> FromIterator<(K, V)> for IndexMap<K, V> {
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        let iter = iter.into_iter();
        let mut map = Self::new();
        let (lo, hi) = iter.size_hint();
        map.values.reserve(hi.unwrap_or(lo));
        for (key, value) in iter {
            map.insert(key, value);
        }
        map
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

pub struct IterMut<'a, K, V> {
    _keys: PhantomData<K>,
    base: Enumerate<slice::IterMut<'a, Option<V>>>,
}

impl<'a, K: From<usize>, V> Iterator for IterMut<'a, K, V> {
    type Item = (K, &'a mut V);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let (index, Some(value)) = self.base.next()? {
                break Some((index.into(), value));
            }
        }
    }
}

pub struct IntoIter<K, V> {
    _keys: PhantomData<K>,
    base: Enumerate<vec::IntoIter<Option<V>>>,
}

impl<K: From<usize>, V> Iterator for IntoIter<K, V> {
    type Item = (K, V);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let (index, Some(value)) = self.base.next()? {
                break Some((index.into(), value));
            }
        }
    }
}

pub struct Drain<'a, K, V> {
    _keys: PhantomData<K>,
    base: Enumerate<vec::Drain<'a, Option<V>>>,
}

impl<K: From<usize>, V> Iterator for Drain<'_, K, V> {
    type Item = (K, V);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let (index, Some(value)) = self.base.next()? {
                break Some((index.into(), value));
            }
        }
    }
}

pub struct Values<'a, V> {
    base: slice::Iter<'a, Option<V>>,
}

impl<'a, V> Iterator for Values<'a, V> {
    type Item = &'a V;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(value) = self.base.next()? {
                break Some(value);
            }
        }
    }
}

pub struct ValuesMut<'a, V> {
    base: slice::IterMut<'a, Option<V>>,
}

impl<'a, V> Iterator for ValuesMut<'a, V> {
    type Item = &'a mut V;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(value) = self.base.next()? {
                break Some(value);
            }
        }
    }
}
