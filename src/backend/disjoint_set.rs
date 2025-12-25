use std::collections::BTreeMap;
use std::fmt::Debug;

type Array<T> = Box<[T]>;

pub struct DisjointSet {
    parent_idx: Array<usize>,
    group_size: Array<usize>,
}

const NO_PARENT: usize = usize::MAX;
impl DisjointSet {
    pub fn new(capacity: usize) -> Self {
        debug_assert!(capacity < usize::MAX);
        DisjointSet {
            parent_idx: vec![NO_PARENT; capacity].into_boxed_slice(),
            group_size: vec![0; capacity].into_boxed_slice(),
        }
    }

    pub fn find_root(&mut self, idx: usize) -> Option<usize> {
        let parent_idx = self.parent_idx[idx];
        if parent_idx == NO_PARENT {
            if self.group_size[idx] == 0 {
                return None;
            } else {
                return Some(idx);
            }
        } else {
            let root = self.find_root(parent_idx);
            if let Some(root_idx) = root {
                self.parent_idx[idx] = root_idx; // 路径压缩
            }
            return root;
        }
    }

    pub fn run_path_compression(&mut self) {
        for idx in 0..self.parent_idx.len() {
            self.find_root(idx);
        }
    }

    pub fn group_size(&mut self, idx: usize) -> usize {
        let root = self.find_root(idx);
        if let Some(root_idx) = root {
            return self.group_size[root_idx];
        } else {
            return 0;
        }
    }

    /// 返回一个group内的所有member
    /// 最好的情况时间复杂度 O(N)
    pub fn group_members(&mut self, idx: usize) -> Option<Vec<usize>> {
        let root = self.find_root(idx);
        if root.is_none() {
            return None;
        }

        let root_idx = root.unwrap();
        let mut members: Vec<usize> = vec![];
        for idx in 0..self.parent_idx.len() {
            if self.find_root(idx) == Some(root_idx) {
                members.push(idx);
            }
        }
        debug_assert!(members.len() == self.group_size(root_idx));
        return Some(members);
    }

    pub fn group_roots(&mut self) -> Vec<usize> {
        let mut roots: Vec<usize> = vec![];
        for idx in 0..self.group_size.len() {
            if self.group_size[idx] > 0 {
                roots.push(idx);
            }
        }
        return roots;
    }

    pub fn delete_group(&mut self, idx: usize) {
        let root = self.find_root(idx);
        if root.is_none() {
            panic!();
        }
        let root_idx = root.unwrap();

        for idx in self.group_members(idx).unwrap() {
            // 恢复初始值
            self.parent_idx[idx] = NO_PARENT;
        }
        self.group_size[root_idx] = 0;
    }

    pub fn is_connected(&mut self, idx_a: usize, idx_b: usize) -> bool {
        let root_a = self.find_root(idx_a);
        let root_b = self.find_root(idx_b);
        if let Some(root_idx_a) = root_a
            && let Some(root_idx_b) = root_b
        {
            return root_idx_a == root_idx_b;
        } else {
            return false;
        }
    }

    /// 添加一个元素，自己成组
    pub fn insert(&mut self, idx: usize) {
        assert!(self.parent_idx[idx] == NO_PARENT);
        assert!(self.group_size[idx] == 0);
        self.group_size[idx] = 1;
    }

    pub fn contains(&self, idx: usize) -> bool {
        self.parent_idx[idx] != NO_PARENT || self.group_size[idx] != 0
    }

    pub fn connect(&mut self, idx_a: usize, idx_b: usize) {
        if !self.contains(idx_a) {
            self.insert(idx_a);
        }
        if !self.contains(idx_b) {
            self.insert(idx_b);
        }

        let root_a = self.find_root(idx_a).unwrap();
        let root_b = self.find_root(idx_b).unwrap();
        if root_a != root_b {
            // 把a挂到b下
            let group_size_a = match self.group_size[root_a] {
                0 => 1,
                n => n,
            };
            let group_size_b = match self.group_size[root_b] {
                0 => 1,
                n => n,
            };
            self.group_size[root_b] = group_size_a + group_size_b;
            self.group_size[root_a] = 0;

            self.parent_idx[root_a] = root_b;
        }
    }
}

impl Clone for DisjointSet {
    fn clone(&self) -> Self {
        Self {
            parent_idx: self.parent_idx.clone(),
            group_size: self.group_size.clone(),
        }
    }
}
impl Debug for DisjointSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{:?}", self.parent_idx)?;
        writeln!(f, "{:?}", self.group_size)?;

        // Work on a temporary copy to avoid mutating self
        let mut tmp = self.clone();

        // root_idx -> member_idxs
        let mut groups: BTreeMap<usize, Vec<usize>> = BTreeMap::new();

        for idx in 0..tmp.parent_idx.len() {
            let root = tmp.find_root(idx);
            if let Some(root_idx) = root {
                groups.entry(root_idx).or_default().push(idx);
            }
        }

        writeln!(f, "Groups:")?;
        for (root_idx, member_idxs) in groups {
            writeln!(f, "    root@[{}], members: {:?}", root_idx, member_idxs)?;
        }
        Ok(())
    }
}
