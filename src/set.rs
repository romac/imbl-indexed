use core::fmt;
use std::hash::{BuildHasher, Hash, RandomState};

use crate::map::IndexMap;

pub struct IndexSet<T, S = RandomState> {
    map: IndexMap<T, (), S>,
}

impl<T, S> fmt::Debug for IndexSet<T, S>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_set().entries(self.iter()).finish()
    }
}

impl<T> IndexSet<T> {
    #[inline]
    pub fn new() -> Self {
        Self::with_hasher(<_>::default())
    }
}

impl<T> Default for IndexSet<T> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<T, S> IndexSet<T, S> {
    #[inline]
    pub fn with_hasher(hash_builder: S) -> Self {
        Self {
            map: IndexMap::with_hasher(hash_builder),
        }
    }

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.map.iter().map(|(k, _)| k)
    }
}

impl<T, S> Clone for IndexSet<T, S>
where
    T: Clone,
    S: Clone,
{
    fn clone(&self) -> Self {
        Self {
            map: self.map.clone(),
        }
    }

    fn clone_from(&mut self, other: &Self) {
        self.map.clone_from(&other.map);
    }
}

impl<T, S> IndexSet<T, S>
where
    T: Hash + Eq,
    S: BuildHasher,
{
    pub fn contains(&self, item: &T) -> bool {
        self.map.contains_key(item)
    }
}

impl<T, S> IndexSet<T, S>
where
    T: Clone + Hash + Eq,
    S: Clone + BuildHasher,
{
    pub fn update(&self, item: T) -> Self {
        Self {
            map: self.map.update(item, ()),
        }
    }

    pub fn without(&self, item: &T) -> Self {
        Self {
            map: self.map.without(item),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_set_is_empty() {
        let set: IndexSet<i32> = IndexSet::new();
        assert!(set.iter().next().is_none());
    }

    #[test]
    fn contains_nonexistent_item() {
        let set: IndexSet<i32> = IndexSet::new();
        assert!(!set.contains(&1));
    }

    #[test]
    fn update_empty_set() {
        let set: IndexSet<i32> = IndexSet::new();
        let updated = set.update(1);

        assert!(updated.contains(&1));
        assert!(!set.contains(&1)); // Original set remains unchanged
    }

    #[test]
    fn update_existing_item() {
        let set: IndexSet<i32> = IndexSet::new();
        let set = set.update(1);
        let updated = set.update(1);

        assert!(updated.contains(&1));
        assert!(set.contains(&1));

        // Check that the size hasn't increased (no duplicates)
        assert_eq!(updated.iter().count(), 1);
    }

    #[test]
    fn multiple_updates() {
        let set: IndexSet<i32> = IndexSet::new();
        let set = set.update(1).update(2).update(3);

        assert!(set.contains(&1));
        assert!(set.contains(&2));
        assert!(set.contains(&3));
        assert_eq!(set.iter().count(), 3);
    }

    #[test]
    fn iter_order() {
        let set: IndexSet<i32> = IndexSet::new();
        let set = set.update(1).update(2).update(3);

        let items: Vec<_> = set.iter().collect();
        assert_eq!(items.len(), 3);

        // Assuming insertion order is maintained
        assert_eq!(items[0], &1);
        assert_eq!(items[1], &2);
        assert_eq!(items[2], &3);
    }

    #[test]
    fn with_strings() {
        let set: IndexSet<String> = IndexSet::new();
        let set = set.update("one".to_string()).update("two".to_string());

        assert!(set.contains(&"one".to_string()));
        assert!(set.contains(&"two".to_string()));
        assert!(!set.contains(&"three".to_string()));
    }

    #[test]
    fn update_preserves_existing_items() {
        let set: IndexSet<i32> = IndexSet::new();
        let set = set.update(1).update(2);
        let updated = set.update(3);

        assert!(updated.contains(&1));
        assert!(updated.contains(&2));
        assert!(updated.contains(&3));
    }

    #[test]
    fn complex_type() {
        #[derive(Clone, Hash, Eq, PartialEq, Debug)]
        struct ComplexItem {
            id: i32,
            name: String,
        }

        let set: IndexSet<ComplexItem> = IndexSet::new();
        let item = ComplexItem {
            id: 1,
            name: "test".to_string(),
        };
        let set = set.update(item.clone());

        assert!(set.contains(&item));
    }

    #[test]
    fn zero_sized_types() {
        #[derive(Clone, Hash, Eq, PartialEq)]
        struct Empty;

        let set: IndexSet<Empty> = IndexSet::new();
        let set = set.update(Empty);

        assert!(set.contains(&Empty));
    }

    #[test]
    fn update_maintains_uniqueness() {
        let set: IndexSet<i32> = IndexSet::new();
        let set = set.update(1).update(1).update(1);

        assert!(set.contains(&1));
        assert_eq!(set.iter().count(), 1);
    }

    #[test]
    fn large_updates() {
        let set: IndexSet<i32> = IndexSet::new();
        let mut set = set;
        for i in 0..1000 {
            set = set.update(i);
        }

        assert_eq!(set.iter().count(), 1000);
        for i in 0..1000 {
            assert!(set.contains(&i));
        }
    }
}
