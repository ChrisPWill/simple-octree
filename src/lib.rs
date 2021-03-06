// Linting
#![warn(rust_2018_idioms)]
#![deny(clippy::all)]
#![warn(clippy::pedantic, clippy::nursery)]
#![allow(clippy::module_name_repetitions)]

#[cfg(test)]
#[macro_use]
extern crate approx;

mod managed_octree;

pub use managed_octree::{
    ManagedHashMapOctree,
    ManagedOctree,
    ManagedOctreeData,
    ManagedVecOctree,
};
use std::{
    borrow::{Borrow, BorrowMut},
    convert::{AsMut, AsRef},
};

/// A barebones octree offering just the methods required for accessing and
/// modifying its contents. Other management structures/functions will be needed
/// to make this more useful, especially for the purpose of querying contents.
#[derive(Default)]
pub struct Octree<D>
where
    D: Default,
{
    children: [Option<Box<Octree<D>>>; 8],
    data: D,
}

#[derive(Debug)]
pub enum AddChildError {
    AlreadyAdded,
    OutOfBoundsIdx,
}

impl<D> Octree<D>
where
    D: Default,
{
    #[must_use]
    pub fn new() -> Self { Self::default() }

    #[must_use]
    pub fn new_with_data(data: D) -> Self {
        Self {
            data,
            ..Self::default()
        }
    }

    /// Adds and returns a reference to a child at a particular index.
    ///
    /// # Errors
    /// Returns an error if the idx is out of range (i.e. idx >= 8) or if the
    /// child is already added.
    pub fn add_child(
        &mut self,
        idx: usize,
        child: Self,
    ) -> Result<&mut Self, AddChildError> {
        if idx >= self.children.len() {
            Err(AddChildError::OutOfBoundsIdx)
        } else if self.children[idx].is_some() {
            Err(AddChildError::AlreadyAdded)
        } else {
            self.children[idx] = Some(Box::new(child));
            self.get_child_mut(idx).ok_or(AddChildError::OutOfBoundsIdx)
        }
    }

    /// Adds and returns a reference to a child at an index based on whether the
    /// child is at the positive or negative side of each axis.
    ///
    /// # Arguments
    /// * `pos_x` - positive x axis if true, negative if false.
    /// * `pos_y` - positive y axis if true, negative if false.
    /// * `pos_z` - positive z axis if true, negative if false.
    ///
    /// # Errors
    /// Returns an error if the child is already added.
    pub fn add_child_at_pos(
        &mut self,
        pos_x: bool,
        pos_y: bool,
        pos_z: bool,
        child: Self,
    ) -> Result<&mut Self, AddChildError> {
        self.add_child(Self::get_child_idx_at_pos(pos_x, pos_y, pos_z), child)
    }

    /// Removes a child and returns the owned value, if it exists.
    pub fn remove_child(&mut self, idx: usize) -> Option<Self> {
        if self.children.get(idx).is_none() {
            None
        } else {
            self.children[idx].take().map(|c| *c)
        }
    }

    /// Removes a child at an index based on whether the child is at the
    /// positive or negative side of each access and returns the owned value, if
    /// it exists.
    ///
    /// # Arguments
    /// * `pos_x` - positive x axis if true, negative if false.
    /// * `pos_y` - positive y axis if true, negative if false.
    /// * `pos_z` - positive z axis if true, negative if false.
    pub fn remove_child_at_pos(
        &mut self,
        pos_x: bool,
        pos_y: bool,
        pos_z: bool,
    ) -> Option<Self> {
        self.remove_child(Self::get_child_idx_at_pos(pos_x, pos_y, pos_z))
    }

    /// Gets a reference to a child given an index.
    #[must_use]
    pub fn get_child(&self, idx: usize) -> Option<&Self> {
        if idx >= self.children.len() {
            None
        } else {
            self.children[idx].as_ref().map(AsRef::as_ref)
        }
    }

    /// Gets a mutable reference to a child given an index.
    #[must_use]
    pub fn get_child_mut(&mut self, idx: usize) -> Option<&mut Self> {
        if idx >= self.children.len() {
            None
        } else {
            self.children[idx].as_mut().map(AsMut::as_mut)
        }
    }

    /// Gets a child index given whether the child is at the positive or
    /// negative side of an axis.
    ///
    /// ## Arguments
    /// * `pos_x` - positive x axis if true, negative if false.
    /// * `pos_y` - positive y axis if true, negative if false.
    /// * `pos_z` - positive z axis if true, negative if false.
    fn get_child_idx_at_pos(pos_x: bool, pos_y: bool, pos_z: bool) -> usize {
        match (pos_x, pos_y, pos_z) {
            (false, false, false) => 0,
            (false, false, true) => 1,
            (false, true, false) => 2,
            (false, true, true) => 3,
            (true, false, false) => 4,
            (true, false, true) => 5,
            (true, true, false) => 6,
            (true, true, true) => 7,
        }
    }

    /// Panics if idx > 7
    fn get_child_pos_at_idx(idx: usize) -> (bool, bool, bool) {
        match idx {
            0 => (false, false, false),
            1 => (false, false, true),
            2 => (false, true, false),
            3 => (false, true, true),
            4 => (true, false, false),
            5 => (true, false, true),
            6 => (true, true, false),
            7 => (true, true, true),
            _ => panic!("idx > 7"),
        }
    }

    /// Gets a reference to a child given whether the child is at the positive
    /// or negative side of an axis.
    ///
    /// ## Arguments
    /// * `pos_x` - positive x axis if true, negative if false.
    /// * `pos_y` - positive y axis if true, negative if false.
    /// * `pos_z` - positive z axis if true, negative if false.
    #[must_use]
    pub fn get_child_at_pos(
        &self,
        pos_x: bool,
        pos_y: bool,
        pos_z: bool,
    ) -> Option<&Self> {
        self.get_child(Self::get_child_idx_at_pos(pos_x, pos_y, pos_z))
    }

    /// Gets a mutable reference to a child given whether the child is at the
    /// positive or negative side of an axis.
    ///
    /// ## Arguments
    /// * `pos_x` - positive x axis if true, negative if false.
    /// * `pos_y` - positive y axis if true, negative if false.
    /// * `pos_z` - positive z axis if true, negative if false.
    #[must_use]
    pub fn get_child_mut_at_pos(
        &mut self,
        pos_x: bool,
        pos_y: bool,
        pos_z: bool,
    ) -> Option<&mut Self> {
        self.get_child_mut(Self::get_child_idx_at_pos(pos_x, pos_y, pos_z))
    }

    /// Gets a reference to the underlying data in the node.
    #[must_use]
    pub fn get_data(&self) -> &D { self.data.borrow() }

    /// Gets a mutable reference to the underlying data in the node.
    #[must_use]
    pub fn get_data_mut(&mut self) -> &mut D { self.data.borrow_mut() }
}

#[cfg(test)]
mod tests {
    use super::Octree;

    #[test]
    fn test_get_child_out_of_bounds_initial() {
        let o = Octree::<Vec<(f32, f32, f32)>>::new();
        assert!(o.get_child(999).is_none());
    }

    #[test]
    fn test_get_child_initial() {
        let o = Octree::<Vec<(f32, f32, f32)>>::new();
        assert!(o.get_child(0).is_none());
    }

    #[test]
    fn test_get_child_pos_initial() {
        let o = Octree::<Vec<(f32, f32, f32)>>::new();
        assert!(o.get_child_at_pos(false, false, false).is_none());
    }

    #[test]
    fn test_add_child() {
        let mut o = Octree::<Vec<(f32, f32, f32)>>::new();
        let result = o.add_child(0, Octree::new());
        assert!(result.is_ok());
    }

    #[test]
    fn test_add_child_at_pos() {
        let mut o = Octree::<Vec<(f32, f32, f32)>>::new();
        let result = o.add_child_at_pos(false, false, false, Octree::new());
        assert!(result.is_ok());
    }

    #[test]
    fn test_remove_child() {
        let mut o = Octree::<Vec<(f32, f32, f32)>>::new();
        o.add_child(0, Octree::new()).unwrap();
        let result = o.remove_child(0);
        assert!(result.is_some());
        assert!(o.get_child(0).is_none());
    }

    #[test]
    fn test_remove_child_at_pos() {
        let mut o = Octree::<Vec<(f32, f32, f32)>>::new();
        o.add_child_at_pos(false, false, false, Octree::new())
            .unwrap();
        let result = o.remove_child_at_pos(false, false, false);
        assert!(result.is_some());
        assert!(o.get_child_at_pos(false, false, false).is_none());
    }
}
