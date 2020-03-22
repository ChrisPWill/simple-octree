use super::Octree;
use len_trait::{Empty, Len};
use num::One;
use std::{
    borrow::{Borrow, BorrowMut},
    collections::HashMap,
    hash::Hash,
};

pub type ManagedOctree<D, S> = Octree<ManagedOctreeData<D, S>>;
pub type ManagedVecOctree<T, S> = ManagedOctree<Vec<T>, S>;
pub type ManagedHashMapOctree<K, V, S> = ManagedOctree<HashMap<K, V>, S>;

pub struct ManagedOctreeData<D, S>
where
    D: Default + Empty + Len,
    S: Default + One,
{
    centre: (S, S, S),
    half_length: S,
    len: usize,
    data: D,
}

impl<D, S> Default for ManagedOctreeData<D, S>
where
    D: Default + Empty + Len,
    S: Default + One,
{
    fn default() -> Self {
        Self {
            centre: (S::default(), S::default(), S::default()),
            half_length: S::one(),
            len: 0,
            data: D::default(),
        }
    }
}

impl<D, S> ManagedOctreeData<D, S>
where
    D: Default + Empty + Len,
    S: Default + One,
{
    /// Gets a reference to the underlying data in the node.
    #[must_use]
    pub fn get_data(&self) -> &D { self.data.borrow() }

    /// Gets a mutable reference to the underlying data in the node.
    #[must_use]
    pub fn get_data_mut(&mut self) -> &mut D { self.data.borrow_mut() }
}

impl<T, S> ManagedVecOctree<T, S>
where
    S: Default + One,
{
    #[must_use]
    pub fn new_managed(centre: (S, S, S), half_length: S) -> Self {
        Self::new_with_data(ManagedOctreeData {
            centre,
            half_length,
            ..ManagedOctreeData::default()
        })
    }
}

impl<K, V, S> ManagedHashMapOctree<K, V, S>
where
    K: Eq + Hash,
    S: Default + One,
{
    #[must_use]
    pub fn new_managed(centre: (S, S, S), half_length: S) -> Self {
        Self::new_with_data(ManagedOctreeData {
            centre,
            half_length,
            ..ManagedOctreeData::default()
        })
    }
}

impl<T, S> Empty for ManagedVecOctree<T, S>
where
    S: Default + One,
{
    fn is_empty(&self) -> bool { self.data.len == 0 }
}

impl<T, S> Len for ManagedVecOctree<T, S>
where
    S: Default + One,
{
    fn len(&self) -> usize { self.data.len }
}

impl<T, S> ManagedVecOctree<T, S>
where
    S: Default + One,
{
    /// Adds data to the node without flushing/rebalancing the tree.
    pub fn add(&mut self, item: T) {
        self.data.data.push(item);
        self.data.len += 1;
    }

    /// Clears data from the node (not the whole tree)
    pub fn clear_data(&mut self) {
        self.data.len -= self.data.data.len();
        self.data.data.clear()
    }
}

impl<K, V, S> Empty for ManagedHashMapOctree<K, V, S>
where
    K: Eq + Hash,
    S: Default + One,
{
    fn is_empty(&self) -> bool { self.data.len == 0 }
}

impl<K, V, S> Len for ManagedHashMapOctree<K, V, S>
where
    K: Eq + Hash,
    S: Default + One,
{
    fn len(&self) -> usize { self.data.len }
}

impl<K, V, S> ManagedHashMapOctree<K, V, S>
where
    K: Eq + Hash,
    S: Default + One,
{
    /// Adds data to the node without flushing/rebalancing the tree.
    pub fn add(&mut self, (key, value): (K, V)) {
        if self.data.data.insert(key, value).is_none() {
            self.data.len += 1;
        }
    }

    /// Clears data from the node (not the whole tree)
    pub fn clear_data(&mut self) {
        self.data.len -= self.data.data.len();
        self.data.data.clear()
    }
}

#[cfg(test)]
mod tests {
    use super::{ManagedHashMapOctree, ManagedVecOctree};
    use len_trait::Len;

    #[test]
    fn test_vec_add() {
        let mut o =
            ManagedVecOctree::<f32, f32>::new_managed((0.0, 0.0, 0.0), 1000.0);
        assert_eq!(o.len(), 0);
        o.add(123.45);
        assert_eq!(o.len(), 1);
    }

    #[test]
    fn test_hash_add() {
        let mut o = ManagedHashMapOctree::<u32, f32, f32>::new_managed(
            (0.0, 0.0, 0.0),
            1000.0,
        );
        assert_eq!(o.len(), 0);
        o.add((123, 456.789));
        assert_eq!(o.len(), 1);
    }
}
