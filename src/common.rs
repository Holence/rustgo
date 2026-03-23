pub type Array<T> = Box<[T]>;

mod coord;
mod disjoint_set;
mod stone;

pub use coord::Coord;
pub use disjoint_set::DisjointSet;
pub use disjoint_set::IdxTrait;
pub use stone::Stone;
