use core::fmt;
use std::hash::{BuildHasher, Hash, Hasher, RandomState};

use imbl::shared_ptr::RcK;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct HashValue(usize);

impl HashValue {
    #[inline(always)]
    fn get(self) -> u64 {
        self.0 as u64
    }
}

impl Hash for HashValue {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.get().hash(state);
    }
}

#[derive(Copy, Clone, Debug)]
struct Bucket<K, V> {
    key: K,
    value: V,
}

type Indices<S> = imbl::GenericHashMap<HashValue, usize, S, RcK>;
type Entries<K, V> = imbl::GenericVector<Option<Bucket<K, V>>, RcK>;

pub struct IndexMap<K, V, S = RandomState> {
    indices: Indices<S>,
    entries: Entries<K, V>,
    hash_builder: S,
}

impl<K, V, S> fmt::Debug for IndexMap<K, V, S>
where
    K: fmt::Debug,
    V: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_map().entries(self.iter()).finish()
    }
}

impl<K, V, S> PartialEq for IndexMap<K, V, S>
where
    K: Hash + Eq,
    V: PartialEq,
    S: BuildHasher,
{
    fn eq(&self, other: &Self) -> bool {
        self.len() == other.len() && self.iter().all(|(k, v)| other.get(k) == Some(v))
    }
}

impl<K, V, S> Eq for IndexMap<K, V, S>
where
    K: Hash + Eq,
    V: PartialEq,
    S: BuildHasher,
{
}

impl<K, V, S> FromIterator<(K, V)> for IndexMap<K, V, S>
where
    S: Clone + Default + BuildHasher,
    K: Clone + Hash + Eq,
    V: Clone,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
    {
        let mut set = Self::new();
        for (key, value) in iter {
            set.insert(key, value);
        }
        set
    }
}

impl<K, V, S> IntoIterator for IndexMap<K, V, S>
where
    K: Clone,
    V: Clone,
{
    type Item = (K, V);
    type IntoIter = IntoIter<K, V>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            entries: self.entries.into_iter(),
        }
    }
}

pub struct IntoIter<K, V> {
    entries: imbl::vector::ConsumingIter<Option<Bucket<K, V>>, RcK>,
}

impl<K, V> Iterator for IntoIter<K, V>
where
    K: Clone,
    V: Clone,
{
    type Item = (K, V);

    fn next(&mut self) -> Option<Self::Item> {
        self.entries.next().flatten().map(|b| (b.key, b.value))
    }
}

impl<K, V, S> IndexMap<K, V, S>
where
    S: Clone + Default,
{
    #[inline]
    pub fn new() -> Self {
        Self::with_hasher(S::default())
    }

    pub fn new_from(&self) -> Self {
        Self {
            indices: Indices::new(),
            entries: Entries::new(),
            hash_builder: self.hash_builder.clone(),
        }
    }
}

impl<K, V, S> Default for IndexMap<K, V, S>
where
    S: Clone + Default,
{
    #[inline]
    fn default() -> Self {
        Self::with_hasher(S::default())
    }
}

impl<K, V, S> IndexMap<K, V, S> {
    #[inline]
    pub fn with_hasher(hash_builder: S) -> Self
    where
        S: Clone,
    {
        Self {
            indices: Indices::with_hasher(hash_builder.clone()),
            entries: Entries::new(),
            hash_builder,
        }
    }

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        self.entries.iter().flatten().map(|b| (&b.key, &b.value))
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl<K, V, S> Clone for IndexMap<K, V, S>
where
    K: Clone,
    V: Clone,
    S: Clone,
{
    fn clone(&self) -> Self {
        IndexMap {
            indices: self.indices.clone(),
            entries: self.entries.clone(),
            hash_builder: self.hash_builder.clone(),
        }
    }

    fn clone_from(&mut self, other: &Self) {
        self.indices.clone_from(&other.indices);
        self.entries.clone_from(&other.entries);
        self.hash_builder.clone_from(&other.hash_builder);
    }
}

impl<K, V, S> IndexMap<K, V, S>
where
    K: Hash + Eq,
    S: BuildHasher,
{
    fn hash(&self, key: &K) -> HashValue {
        HashValue(self.hash_builder.hash_one(key) as usize)
    }
}

impl<K, V, S> IndexMap<K, V, S>
where
    K: Hash + Eq,
    S: BuildHasher,
{
    pub fn get(&self, key: &K) -> Option<&V> {
        let hash = self.hash(key);
        self.indices
            .get(&hash)
            .and_then(|idx| self.entries.get(*idx))
            .and_then(|e| e.as_ref())
            .filter(|b| b.key == *key)
            .map(|b| &b.value)
    }

    pub fn contains_key(&self, key: &K) -> bool {
        let hash = self.hash(key);
        self.indices
            .get(&hash)
            .and_then(|idx| self.entries.get(*idx))
            .and_then(|e| e.as_ref())
            .filter(|b| b.key == *key)
            .is_some()
    }
}

impl<K, V, S> IndexMap<K, V, S>
where
    K: Clone + Hash + Eq,
    V: Clone,
    S: Clone + BuildHasher,
{
    pub fn insert(&mut self, key: K, value: V) {
        let hash = self.hash(&key);
        let bucket = Bucket { key, value };

        if let Some(idx) = self.indices.get(&hash) {
            self.entries.get_mut(*idx).unwrap().replace(bucket);
        } else {
            let idx = self.entries.len();
            self.indices.insert(hash, idx);
            self.entries.push_back(Some(bucket));
        }
    }

    pub fn update(&self, key: K, value: V) -> Self {
        let hash = self.hash(&key);
        let bucket = Some(Bucket { key, value });

        if let Some(idx) = self.indices.get(&hash) {
            let entries = self.entries.update(*idx, bucket);
            Self {
                indices: self.indices.clone(),
                entries,
                hash_builder: self.hash_builder.clone(),
            }
        } else {
            let idx = self.entries.len();
            let indices = self.indices.update(hash, idx);
            let mut entries = self.entries.clone();
            entries.push_back(bucket);

            Self {
                indices,
                entries,
                hash_builder: self.hash_builder.clone(),
            }
        }
    }

    pub fn without(&self, key: &K) -> Self {
        let hash = self.hash(key);

        if let Some(idx) = self.indices.get(&hash).copied() {
            let indices = self.indices.without(&hash);
            let entries = self.entries.update(idx, None);

            Self {
                indices,
                entries,
                hash_builder: self.hash_builder.clone(),
            }
        } else {
            self.clone()
        }
    }

    pub fn remove(&mut self, key: &K) {
        let hash = self.hash(key);

        if let Some(idx) = self.indices.get(&hash).copied() {
            self.indices.remove(&hash);
            self.entries.remove(idx);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type IndexMap<K, V> = super::IndexMap<K, V, RandomState>;

    #[test]
    fn new_map_is_empty() {
        let map: IndexMap<i32, String> = IndexMap::new();
        assert!(map.iter().next().is_none());
    }

    #[test]
    fn get_nonexistent_key() {
        let map: IndexMap<i32, String> = IndexMap::new();
        assert_eq!(map.get(&1), None);
    }

    #[test]
    fn update_empty_map() {
        let map: IndexMap<i32, String> = IndexMap::new();
        let updated = map.update(1, "one".to_string());

        assert_eq!(updated.get(&1), Some(&"one".to_string()));
        assert!(map.get(&1).is_none()); // Original map remains unchanged
    }

    #[test]
    fn update_existing_key() {
        let map: IndexMap<i32, String> = IndexMap::new();
        let map = map.update(1, "one".to_string());
        let updated = map.update(1, "new one".to_string());

        assert_eq!(updated.get(&1), Some(&"new one".to_string()));
        assert_eq!(map.get(&1), Some(&"one".to_string())); // Original map remains unchanged
    }

    #[test]
    fn multiple_updates() {
        let map: IndexMap<i32, String> = IndexMap::new();
        let map = map.update(1, "one".to_string());
        let map = map.update(2, "two".to_string());
        let map = map.update(3, "three".to_string());

        assert_eq!(map.get(&1), Some(&"one".to_string()));
        assert_eq!(map.get(&2), Some(&"two".to_string()));
        assert_eq!(map.get(&3), Some(&"three".to_string()));
    }

    #[test]
    fn insert_empty_map() {
        let mut map: IndexMap<i32, String> = IndexMap::new();
        map.insert(1, "one".to_string());

        assert_eq!(map.get(&1), Some(&"one".to_string()));
    }

    #[test]
    fn insert_existing_key() {
        let mut map: IndexMap<i32, String> = IndexMap::new();
        map.insert(1, "one".to_string());
        map.insert(1, "new one".to_string());

        assert_eq!(map.get(&1), Some(&"new one".to_string()));
    }

    #[test]
    fn multiple_inserts() {
        let mut map: IndexMap<i32, String> = IndexMap::new();
        map.insert(1, "one".to_string());
        map.insert(2, "two".to_string());
        map.insert(3, "three".to_string());

        assert_eq!(map.get(&1), Some(&"one".to_string()));
        assert_eq!(map.get(&2), Some(&"two".to_string()));
        assert_eq!(map.get(&3), Some(&"three".to_string()));
    }

    #[test]
    fn iter_order() {
        let map: IndexMap<i32, String> = IndexMap::new();
        let map = map
            .update(1, "one".to_string())
            .update(2, "two".to_string())
            .update(3, "three".to_string());

        let items: Vec<_> = map.iter().collect();
        assert_eq!(items.len(), 3);

        // Assuming insertion order is maintained
        assert_eq!(items[0], (&1, &"one".to_string()));
        assert_eq!(items[1], (&2, &"two".to_string()));
        assert_eq!(items[2], (&3, &"three".to_string()));
    }

    #[test]
    fn with_string_keys() {
        let map: IndexMap<String, i32> = IndexMap::new();
        let map = map
            .update("one".to_string(), 1)
            .update("two".to_string(), 2);

        assert_eq!(map.get(&"one".to_string()), Some(&1));
        assert_eq!(map.get(&"two".to_string()), Some(&2));
    }

    #[test]
    fn update_preserves_other_values() {
        let map: IndexMap<i32, String> = IndexMap::new();
        let map = map
            .update(1, "one".to_string())
            .update(2, "two".to_string());
        let updated = map.update(1, "new one".to_string());

        assert_eq!(updated.get(&1), Some(&"new one".to_string()));
        assert_eq!(updated.get(&2), Some(&"two".to_string())); // Other values preserved
    }

    #[test]
    fn update_complex_type() {
        #[derive(Clone, Hash, Eq, PartialEq)]
        struct ComplexKey {
            id: i32,
            name: String,
        }

        let map: IndexMap<ComplexKey, Vec<i32>> = IndexMap::new();
        let key = ComplexKey {
            id: 1,
            name: "test".to_string(),
        };
        let map = map.update(key.clone(), vec![1, 2, 3]);

        assert_eq!(map.get(&key), Some(&vec![1, 2, 3]));
    }

    #[test]
    fn update_zero_sized_types() {
        #[derive(Clone, Hash, Eq, PartialEq)]
        struct Empty;

        let map: IndexMap<Empty, Empty> = IndexMap::new();
        let map = map.update(Empty, Empty);

        assert!(map.get(&Empty).is_some());
    }

    #[test]
    fn test_without_from_empty_map() {
        let map: IndexMap<i32, String> = IndexMap::new();
        let result = map.without(&1);
        assert!(result.get(&1).is_none());
        assert_eq!(result.iter().count(), 0);
    }

    #[test]
    fn test_without_existing_key() {
        let map = IndexMap::new()
            .update(1, "one".to_string())
            .update(2, "two".to_string());

        let result = map.without(&1);

        assert!(result.get(&1).is_none());
        assert_eq!(result.get(&2), Some(&"two".to_string()));
        assert_eq!(result.iter().count(), 1);

        // Original map should remain unchanged
        assert_eq!(map.get(&1), Some(&"one".to_string()));
        assert_eq!(map.get(&2), Some(&"two".to_string()));
    }

    #[test]
    fn test_without_nonexistent_key() {
        let map = IndexMap::new().update(1, "one".to_string());

        let result = map.without(&2);

        assert_eq!(result.get(&1), Some(&"one".to_string()));
        assert_eq!(result.iter().count(), 1);
    }

    #[test]
    fn test_without_preserves_order() {
        let map = IndexMap::new()
            .update(1, "one".to_string())
            .update(2, "two".to_string())
            .update(3, "three".to_string())
            .update(4, "four".to_string());

        let result = map.without(&2);

        let items: Vec<_> = result.iter().collect();
        assert_eq!(items.len(), 3);
        assert_eq!(items[0], (&1, &"one".to_string()));
        assert_eq!(items[1], (&3, &"three".to_string()));
        assert_eq!(items[2], (&4, &"four".to_string()));
    }

    #[test]
    fn test_without_chaining() {
        let map = IndexMap::new()
            .update(1, "one".to_string())
            .update(2, "two".to_string())
            .update(3, "three".to_string());

        let result = map.without(&1).without(&2);

        assert!(result.get(&1).is_none());
        assert!(result.get(&2).is_none());
        assert_eq!(result.get(&3), Some(&"three".to_string()));
        assert_eq!(result.iter().count(), 1);
    }

    #[test]
    fn test_without_all_elements() {
        let map = IndexMap::new()
            .update(1, "one".to_string())
            .update(2, "two".to_string());

        let result = map.without(&1).without(&2);

        assert_eq!(result.iter().count(), 0);
    }

    #[test]
    fn test_without_with_string_keys() {
        let map = IndexMap::new()
            .update("one".to_string(), 1)
            .update("two".to_string(), 2);

        let result = map.without(&"one".to_string());

        assert!(result.get(&"one".to_string()).is_none());
        assert_eq!(result.get(&"two".to_string()), Some(&2));
    }

    #[test]
    fn test_without_complex_key() {
        #[derive(Clone, Hash, Eq, PartialEq, Debug)]
        struct ComplexKey {
            id: i32,
            name: String,
        }

        let key1 = ComplexKey {
            id: 1,
            name: "first".to_string(),
        };
        let key2 = ComplexKey {
            id: 2,
            name: "second".to_string(),
        };

        let map = IndexMap::new()
            .update(key1.clone(), "value1".to_string())
            .update(key2.clone(), "value2".to_string());

        let result = map.without(&key1);

        assert!(result.get(&key1).is_none());
        assert_eq!(result.get(&key2), Some(&"value2".to_string()));
    }
}
