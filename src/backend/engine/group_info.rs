use std::ops::{Index, IndexMut};

use crate::backend::{Array, Idx};

// TODO more assert on this info
#[derive(Clone, Debug)]
pub struct GroupInfo {
    pub qi: usize,         // group的气
    pub members: Vec<Idx>, // group的所有棋子
}

impl GroupInfo {
    pub fn new(qi: usize, members: Vec<Idx>) -> Self {
        Self { qi, members }
    }
}

pub struct GroupInfoArray(Array<Option<Box<GroupInfo>>>);
impl GroupInfoArray {
    pub fn new(size: usize) -> Self {
        GroupInfoArray(vec![None; size].into_boxed_slice())
    }

    pub fn get(&self, index: usize) -> &GroupInfo {
        self.0[index].as_ref().unwrap()
    }

    pub fn get_mut(&mut self, index: usize) -> &mut GroupInfo {
        self.0[index].as_mut().unwrap()
    }
}

impl Index<usize> for GroupInfoArray {
    type Output = Option<Box<GroupInfo>>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl IndexMut<usize> for GroupInfoArray {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}
