use std::convert::{AsMut, AsRef};

pub struct Octree<T> {
    children: [Option<Box<Octree<T>>>; 8],
    objects: Vec<T>,
}

impl<T> Octree<T> {
    fn get_child_idx(pos_x: bool, pos_y: bool, pos_z: bool) -> usize {
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

    #[must_use]
    pub fn get_child(
        &self,
        pos_x: bool,
        pos_y: bool,
        pos_z: bool,
    ) -> Option<&Self> {
        self.children[Self::get_child_idx(pos_x, pos_y, pos_z)]
            .as_ref()
            .map(AsRef::as_ref)
    }

    #[must_use]
    pub fn get_child_mut(
        &mut self,
        pos_x: bool,
        pos_y: bool,
        pos_z: bool,
    ) -> Option<&mut Self> {
        self.children[Self::get_child_idx(pos_x, pos_y, pos_z)]
            .as_mut()
            .map(AsMut::as_mut)
    }

    #[must_use]
    pub const fn get_objects(&self) -> &Vec<T> { &self.objects }

    #[must_use]
    pub fn get_objects_mut(&mut self) -> &mut Vec<T> { &mut self.objects }
}
