use std::borrow::Borrow;
use std::collections::btree_map::{Iter, IterMut, Range, RangeMut};
use std::collections::{BTreeMap, VecDeque};
use std::ops::RangeBounds;

#[derive(Debug, Clone)]
pub struct BTreeKeyValues<K, V>
where
    K: Ord,
{
    btree: BTreeMap<K, VecDeque<V>>,
}

impl<K, V> Default for BTreeKeyValues<K, V>
where
    K: Ord,
{
    fn default() -> BTreeKeyValues<K, V> {
        BTreeKeyValues {
            btree: BTreeMap::new(),
        }
    }
}

impl<K, V> BTreeKeyValues<K, V>
where
    K: Ord,
{
    pub fn push_back(&mut self, key: K, value: V)
    where
        K: Ord + Copy,
    {
        self.btree.entry(key).or_default().push_back(value);
    }

    pub fn push_front(&mut self, key: K, value: V)
    where
        K: Ord + Copy,
    {
        self.btree.entry(key).or_default().push_front(value);
    }

    pub fn range<R>(&self, range: R) -> Range<'_, K, VecDeque<V>>
    where
        V: Ord,
        K: Borrow<V>,
        R: RangeBounds<V>,
    {
        self.btree.range(range)
    }

    pub fn range_mut<R>(&mut self, range: R) -> RangeMut<'_, K, VecDeque<V>>
    where
        V: Ord,
        K: Borrow<V>,
        R: RangeBounds<V>,
    {
        self.btree.range_mut(range)
    }

    pub fn pop_first_back(&mut self) -> Option<V>
    where
        K: Ord,
    {
        loop {
            return match self.btree.pop_first() {
                None => None,
                Some((key, mut queue)) => {
                    let ret = match queue.pop_front() {
                        None => continue,
                        Some(x) => x,
                    };
                    if !queue.is_empty() {
                        self.btree.insert(key, queue);
                    }
                    Some(ret)
                }
            };
        }
    }

    pub fn pop_last_back(&mut self) -> Option<V>
    where
        K: Ord,
    {
        match self.btree.pop_last() {
            None => None,
            Some((key, mut queue)) => {
                let ret = match queue.pop_front() {
                    None => panic!("Illegal state"),
                    Some(x) => x,
                };
                if !queue.is_empty() {
                    self.btree.insert(key, queue);
                }
                Some(ret)
            }
        }
    }

    pub fn remove(&mut self, key: &K, value: &V) -> Option<V>
    where
        V: Eq,
    {
        let values = self.btree.get_mut(key)?;
        let index = values.iter().position(|v| v == value)?;
        let ret = values.remove(index);
        if values.is_empty() {
            self.btree.remove(key);
        }
        ret
    }

    pub fn first_key_value(&self) -> Option<(&K, &V)> {
        self.btree
            .first_key_value()
            .and_then(|(key, values)| values.front().map(|value| (key, value)))
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.btree.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.btree.is_empty()
    }

    #[inline]
    pub fn iter(&self) -> Iter<'_, K, VecDeque<V>> {
        self.btree.iter()
    }

    #[inline]
    pub fn iter_mut(&mut self) -> IterMut<'_, K, VecDeque<V>> {
        self.btree.iter_mut()
    }

    #[inline]
    pub fn clear(&mut self) {
        self.btree.clear();
    }
}

impl<K, V> From<BTreeKeyValues<K, V>> for Vec<V>
where
    K: Ord,
{
    fn from(mut map: BTreeKeyValues<K, V>) -> Self {
        let mut ret = vec![];
        while let Some(x) = map.pop_first_back() {
            ret.push(x);
        }
        ret
    }
}
