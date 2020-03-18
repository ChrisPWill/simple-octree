// Linting
#![warn(rust_2018_idioms)]
#![deny(clippy::all)]
#![warn(clippy::pedantic, clippy::nursery)]
#![allow(clippy::module_name_repetitions)]

mod octree;

pub use octree::Octree;
