use super::Octree;
use std::borrow::{Borrow, BorrowMut};

pub type ManagedOctree<D> = Octree<ManagedOctreeData<D>>;

pub struct ManagedOctreeData<D>
where
    D: Default,
{
    data: D,
}

impl<D> Default for ManagedOctreeData<D>
where
    D: Default,
{
    fn default() -> Self { Self { data: D::default() } }
}

impl<D> ManagedOctreeData<D>
where
    D: Default,
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
