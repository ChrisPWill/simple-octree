use super::Octree;
use len_trait::{Clear, Empty, Len};
use num::One;
use std::{
    borrow::{Borrow, BorrowMut},
    collections::HashMap,
    hash::Hash,
    mem,
    ops::{Add, Div, Sub},
};

pub type ManagedOctree<D, S> = Octree<ManagedOctreeData<D, S>>;
pub type ManagedVecOctree<T, S> = ManagedOctree<Vec<T>, S>;
pub type ManagedHashMapOctree<K, V, S> = ManagedOctree<HashMap<K, V>, S>;

/// A trait that will allow the underlying collection to be treated generically.
pub trait OctreeCollection<I> {
    fn add(&mut self, item: I) -> Option<()>;
}

pub trait CentredItem<S> {
    fn centre(&self) -> (S, S, S);
}

impl<S> CentredItem<S> for (S, S, S)
where
    S: Copy,
{
    fn centre(&self) -> (S, S, S) { *self }
}

impl<S, K> CentredItem<S> for (K, (S, S, S))
where
    S: Copy,
{
    fn centre(&self) -> (S, S, S) { self.1 }
}

impl<I> OctreeCollection<I> for Vec<I> {
    fn add(&mut self, item: I) -> Option<()> {
        self.push(item);
        Some(())
    }
}

impl<K, V> OctreeCollection<(K, V)> for HashMap<K, V>
where
    K: Eq + Hash,
{
    fn add(&mut self, (key, val): (K, V)) -> Option<()> {
        if self.contains_key(&key) {
            return None;
        }
        self.insert(key, val);
        Some(())
    }
}

pub struct ManagedOctreeData<D, S>
where
    D: Default + Empty + Len,
    S: Default + One,
{
    centre: (S, S, S),
    half_length: S,
    max_size: usize,
    drop_below_size: usize,
    len: usize,
    data: D,
}

impl<D, S> Default for ManagedOctreeData<D, S>
where
    D: Default + Empty + Len,
    S: Default
        + Copy
        + One
        + Add<S, Output = S>
        + Sub<S, Output = S>
        + Div<S, Output = S>,
{
    fn default() -> Self {
        Self {
            centre: (S::default(), S::default(), S::default()),
            half_length: S::one(),
            max_size: 1,
            drop_below_size: 1,
            len: 0,
            data: D::default(),
        }
    }
}

impl<D, S> ManagedOctreeData<D, S>
where
    D: Default + Empty + Len,
    S: Default
        + Copy
        + One
        + Add<S, Output = S>
        + Sub<S, Output = S>
        + Div<S, Output = S>,
{
    /// Gets a reference to the underlying data in the node.
    #[must_use]
    pub fn get_data(&self) -> &D { self.data.borrow() }

    /// Gets a mutable reference to the underlying data in the node.
    #[must_use]
    pub fn get_data_mut(&mut self) -> &mut D { self.data.borrow_mut() }
}

impl<D, S, T> ManagedOctree<D, S>
where
    D: Default
        + Empty
        + Len
        + Clear
        + IntoIterator<Item = T>
        + OctreeCollection<T>,
    T: CentredItem<S>,
    S: Default
        + Copy
        + One
        + PartialOrd
        + Add<S, Output = S>
        + Sub<S, Output = S>
        + Div<S, Output = S>,
{
    #[must_use]
    pub fn new_managed(centre: (S, S, S), half_length: S) -> Self {
        Self::new_with_data(ManagedOctreeData {
            centre,
            half_length,
            ..ManagedOctreeData::default()
        })
    }

    /// Set `max_size`
    #[must_use]
    pub fn with_max_size(mut self, max_size: usize) -> Self {
        self.data.max_size = max_size;
        self
    }

    /// Set `drop_below_size`
    ///
    /// Panics when set to 0
    #[must_use]
    pub fn with_drop_below_size(mut self, drop_below_size: usize) -> Self {
        if drop_below_size == 0 {
            panic!("drop_below_size must be greater than 0");
        }

        self.data.drop_below_size = drop_below_size;
        self
    }

    /// Adds data to the node without flushing/rebalancing the tree.
    pub fn add(&mut self, item: T) {
        self.data.data.add(item);
        self.data.len += 1;
    }

    /// Clears data from the node (not the whole tree)
    pub fn clear_data(&mut self) {
        self.data.len -= self.data.data.len();
        self.data.data.clear()
    }

    pub fn rebalance(&mut self) {
        let bucket_counts = self.move_to_existing_children();
        if self.data.data.len() <= self.data.max_size {
            return;
        }
        let bucket_sizes = Self::sort_bucket_sizes(bucket_counts);
        let mut new_size = self.data.data.len();
        for (max_idx, max_val) in bucket_sizes {
            let (px, py, pz) = Self::get_child_pos_at_idx(max_idx);
            let (centre, half_length) =
                self.get_child_centre_and_half_length_at_pos(px, py, pz);
            self.add_child(
                max_idx,
                Self::new_managed(centre, half_length)
                    .with_max_size(self.data.max_size)
                    .with_drop_below_size(self.data.drop_below_size),
            )
            .unwrap();
            new_size -= max_val;
            if new_size <= self.data.max_size {
                break;
            }
        }
        self.move_to_existing_children();
    }

    fn sort_bucket_sizes(sizes: [usize; 8]) -> Vec<(usize, usize)> {
        let mut bucket_sizes: Vec<(usize, usize)> =
            sizes.iter().enumerate().map(|(i, &v)| (i, v)).collect();
        bucket_sizes.sort_unstable_by(|(_ai, am), (_bi, bm)| {
            bm.partial_cmp(am).unwrap()
        });
        bucket_sizes
    }

    /// Moves any objects that should belong to a child to that child if it
    /// exists. Returns the bucket sizes of any remaining items.
    fn move_to_existing_children(&mut self) -> [usize; 8] {
        let (cx, cy, cz) = self.data.centre;

        let mut result = [0; 8];
        let mut old_d = D::default();
        mem::swap(&mut old_d, &mut self.data.data);
        for item in old_d {
            let (ix, iy, iz) = item.centre();
            let idx = Self::get_child_idx_at_pos(ix > cx, iy > cy, iz > cz);
            if let Some(child) = &mut self.children[idx] {
                child.add(item);
            } else {
                self.add(item);
                result[idx] += 1;
            }
        }

        result
    }

    fn get_child_centre_and_half_length_at_pos(
        &self,
        pos_x: bool,
        pos_y: bool,
        pos_z: bool,
    ) -> ((S, S, S), S) {
        let (cx, cy, cz) = self.data.centre;
        let hhl = self.data.half_length / (S::one() + S::one());
        match (pos_x, pos_y, pos_z) {
            (false, false, false) => ((cx - hhl, cy - hhl, cz - hhl), (hhl)),
            (false, false, true) => ((cx - hhl, cy - hhl, cz + hhl), (hhl)),
            (false, true, false) => ((cx - hhl, cy + hhl, cz - hhl), (hhl)),
            (false, true, true) => ((cx - hhl, cy + hhl, cz + hhl), (hhl)),
            (true, false, false) => ((cx + hhl, cy - hhl, cz - hhl), (hhl)),
            (true, false, true) => ((cx + hhl, cy - hhl, cz + hhl), (hhl)),
            (true, true, false) => ((cx + hhl, cy + hhl, cz - hhl), (hhl)),
            (true, true, true) => ((cx + hhl, cy + hhl, cz + hhl), (hhl)),
        }
    }
}

impl<T, S> Empty for ManagedVecOctree<T, S>
where
    S: Default
        + Copy
        + One
        + Add<S, Output = S>
        + Sub<S, Output = S>
        + Div<S, Output = S>,
{
    fn is_empty(&self) -> bool { self.data.len == 0 }
}

impl<T, S> Len for ManagedVecOctree<T, S>
where
    S: Default
        + Copy
        + One
        + Add<S, Output = S>
        + Sub<S, Output = S>
        + Div<S, Output = S>,
{
    fn len(&self) -> usize { self.data.len }
}

impl<K, V, S> Empty for ManagedHashMapOctree<K, V, S>
where
    K: Eq + Hash,
    S: Default
        + Copy
        + One
        + Add<S, Output = S>
        + Sub<S, Output = S>
        + Div<S, Output = S>,
{
    fn is_empty(&self) -> bool { self.data.len == 0 }
}

impl<K, V, S> Len for ManagedHashMapOctree<K, V, S>
where
    K: Eq + Hash,
    S: Default
        + Copy
        + One
        + Add<S, Output = S>
        + Sub<S, Output = S>
        + Div<S, Output = S>,
{
    fn len(&self) -> usize { self.data.len }
}

#[cfg(test)]
mod tests {
    use super::{ManagedHashMapOctree, ManagedVecOctree};
    use len_trait::Len;

    #[test]
    fn test_with_drop_below_size() {
        let o = ManagedVecOctree::<(f32, f32, f32), f32>::new_managed(
            (0.0, 0.0, 0.0),
            1000.0,
        )
        .with_drop_below_size(3);
        assert_eq!(o.data.drop_below_size, 3);
    }

    #[test]
    #[should_panic]
    fn test_with_drop_below_size_0_panics() {
        let _ = ManagedVecOctree::<(f32, f32, f32), f32>::new_managed(
            (0.0, 0.0, 0.0),
            1000.0,
        )
        .with_drop_below_size(0);
    }

    #[test]
    fn test_with_max_size() {
        let o = ManagedVecOctree::<(f32, f32, f32), f32>::new_managed(
            (0.0, 0.0, 0.0),
            1000.0,
        )
        .with_max_size(3);
        assert_eq!(o.data.max_size, 3);
    }

    #[test]
    fn test_get_child_centre_and_half_length_neg() {
        let o = ManagedVecOctree::<(f32, f32, f32), f32>::new_managed(
            (0.0, 0.0, 0.0),
            1000.0,
        );
        let ((cx, cy, cz), half_length) =
            o.get_child_centre_and_half_length_at_pos(false, false, false);
        assert_relative_eq!(cx, -500.0);
        assert_relative_eq!(cy, -500.0);
        assert_relative_eq!(cz, -500.0);
        assert_relative_eq!(half_length, 500.0);
    }

    #[test]
    fn test_get_child_centre_and_half_length_pos() {
        let o = ManagedVecOctree::<(f32, f32, f32), f32>::new_managed(
            (0.0, 0.0, 0.0),
            1000.0,
        );
        let ((cx, cy, cz), half_length) =
            o.get_child_centre_and_half_length_at_pos(true, true, true);
        assert_relative_eq!(cx, 500.0);
        assert_relative_eq!(cy, 500.0);
        assert_relative_eq!(cz, 500.0);
        assert_relative_eq!(half_length, 500.0);
    }

    #[test]
    fn test_get_child_centre_and_half_length_partial_pos_off_centre() {
        let o = ManagedVecOctree::<(f32, f32, f32), f32>::new_managed(
            (100.0, 200.0, 300.0),
            1000.0,
        );
        let ((cx, cy, cz), half_length) =
            o.get_child_centre_and_half_length_at_pos(true, false, true);
        assert_relative_eq!(cx, 600.0);
        assert_relative_eq!(cy, -300.0);
        assert_relative_eq!(cz, 800.0);
        assert_relative_eq!(half_length, 500.0);
    }

    #[test]
    fn test_vec_add() {
        let mut o = ManagedVecOctree::<(f32, f32, f32), f32>::new_managed(
            (0.0, 0.0, 0.0),
            1000.0,
        );
        assert_eq!(o.len(), 0);
        o.add((123.45, 234.567, 345.678));
        assert_eq!(o.len(), 1);
    }

    #[test]
    fn test_hash_add() {
        let mut o =
            ManagedHashMapOctree::<u32, (f32, f32, f32), f32>::new_managed(
                (0.0, 0.0, 0.0),
                1000.0,
            );
        assert_eq!(o.len(), 0);
        o.add((123, (123.45, 234.567, 345.678)));
        assert_eq!(o.len(), 1);
    }

    #[test]
    fn test_rebalance_max_2() {
        let mut o = ManagedVecOctree::<(f32, f32, f32), f32>::new_managed(
            (0.0, 0.0, 0.0),
            1000.0,
        )
        .with_max_size(2);
        o.add((1.0, 1.0, 1.0));
        o.add((2.0, 2.0, 1.0));
        o.add((-1.0, -1.0, -1.0));
        o.rebalance();
        assert_eq!(o.data.data.len(), 1);
        assert!(o.get_child_at_pos(true, true, true).is_some());
        assert!(o.get_child_at_pos(false, false, false).is_none());
        assert_eq!(
            o.get_child_at_pos(true, true, true)
                .unwrap()
                .data
                .data
                .len(),
            2
        );
    }
}
