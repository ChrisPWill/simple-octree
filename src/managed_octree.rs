use super::Octree;
use len_trait::{Empty, Len};
use std::borrow::{Borrow, BorrowMut};

pub type ManagedOctree<D> = Octree<ManagedOctreeData<D>>;

pub struct ManagedOctreeData<D>
where
    D: Default + Empty + Len,
{
    data: D,
}

impl<D> Default for ManagedOctreeData<D>
where
    D: Default + Empty + Len,
{
    fn default() -> Self { Self { data: D::default() } }
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

impl<D> Empty for ManagedOctree<D>
where
    D: Default + Empty + Len,
{
    fn is_empty(&self) -> bool {
        self.data.data.is_empty()
            && self
                .children
                .iter()
                .filter_map(Option::as_ref)
                .all(|c| c.is_empty())
    }
}

impl<D> Len for ManagedOctree<D>
where
    D: Default + Empty + Len,
{
    fn len(&self) -> usize {
        let children_sum: usize = self
            .children
            .iter()
            .filter_map(|c| c.as_ref().map(Len::len))
            .sum();
        self.data.data.len() + children_sum
    }
}
