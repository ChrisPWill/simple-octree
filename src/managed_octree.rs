use super::Octree;
use len_trait::{Empty, Len};
use std::{
    borrow::{Borrow, BorrowMut},
    collections::HashMap,
    hash::Hash,
};

pub type ManagedOctree<D> = Octree<ManagedOctreeData<D>>;
pub type ManagedVecOctree<T> = ManagedOctree<Vec<T>>;
pub type ManagedHashMapOctree<K, V> = ManagedOctree<HashMap<K, V>>;

pub struct ManagedOctreeData<D>
where
    D: Default + Empty + Len,
{
    len: usize,
    data: D,
}

impl<D> Default for ManagedOctreeData<D>
where
    D: Default + Empty + Len,
{
    fn default() -> Self {
        Self {
            len: 0,
            data: D::default(),
        }
    }
}

impl<D> ManagedOctreeData<D>
where
    D: Default + Empty + Len,
{
    #[must_use]
    pub fn new() -> Self { Self::default() }

    /// Gets a reference to the underlying data in the node.
    #[must_use]
    pub fn get_data(&self) -> &D { self.data.borrow() }

    /// Gets a mutable reference to the underlying data in the node.
    #[must_use]
    pub fn get_data_mut(&mut self) -> &mut D { self.data.borrow_mut() }
}

impl<T> ManagedVecOctree<T> {
    #[must_use]
    pub fn new_managed() -> Self {
        Self::new_with_data(ManagedOctreeData::new())
    }
}

impl<K, V> ManagedHashMapOctree<K, V>
where
    K: Eq + Hash,
{
    #[must_use]
    pub fn new_managed() -> Self {
        Self::new_with_data(ManagedOctreeData::new())
    }
}

impl<T> Empty for ManagedVecOctree<T> {
    fn is_empty(&self) -> bool { self.data.len == 0 }
}

impl<T> Len for ManagedVecOctree<T> {
    fn len(&self) -> usize { self.data.len }
}

impl<T> ManagedVecOctree<T> {
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

impl<K, V> Empty for ManagedHashMapOctree<K, V>
where
    K: Eq + Hash,
{
    fn is_empty(&self) -> bool { self.data.len == 0 }
}

impl<K, V> Len for ManagedHashMapOctree<K, V>
where
    K: Eq + Hash,
{
    fn len(&self) -> usize { self.data.len }
}

impl<K, V> ManagedHashMapOctree<K, V>
where
    K: Eq + Hash,
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
        let mut o = ManagedVecOctree::<f32>::new_managed();
        assert_eq!(o.len(), 0);
        o.add(123.45);
        assert_eq!(o.len(), 1);
    }

    #[test]
    fn test_hash_add() {
        let mut o = ManagedHashMapOctree::<u32, f32>::new_managed();
        assert_eq!(o.len(), 0);
        o.add((123, 456.789));
        assert_eq!(o.len(), 1);
    }
}
