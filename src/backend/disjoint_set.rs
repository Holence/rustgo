use std::collections::BTreeMap;
use std::fmt::Debug;

use crate::backend::Array;

pub struct DisjointSet {
    // 负数则表示当前 index 是 group root, 负数的绝对值为 group 的 size
    // 正数则表示当前 index 不是 group root, 值表示 parent 的 index
    group_idx: Array<isize>,
}

impl DisjointSet {
    pub fn new(capacity: usize) -> Self {
        DisjointSet {
            group_idx: vec![-1; capacity].into_boxed_slice(),
        }
    }

    pub fn find_root(&mut self, idx: usize) -> usize {
        let parent_idx = self.group_idx[idx];
        if parent_idx < 0 {
            return idx;
        } else {
            let root = self.find_root(parent_idx as usize);
            self.group_idx[idx] = root as isize; // 路径压缩
            return root;
        }
    }

    pub fn group_size(&mut self, idx: usize) -> usize {
        let root = self.find_root(idx);
        return (-self.group_idx[root]) as usize;
    }

    /// 最好的情况时间复杂度 O(N)
    pub fn group_members(&mut self, idx: usize) -> Vec<usize> {
        let root = self.find_root(idx);
        let mut members: Vec<usize> = vec![];
        for idx in 0..self.group_idx.len() {
            if self.find_root(idx) == root {
                members.push(idx);
            }
        }
        debug_assert!(members.len() == self.group_size(root));
        return members;
    }

    pub fn delete_group(&mut self, idx: usize) {
        let members = self.group_members(idx);
        for idx in members {
            // 恢复初始值
            self.group_idx[idx] = -1;
        }
    }

    pub fn is_connected(&mut self, idx_a: usize, idx_b: usize) -> bool {
        self.find_root(idx_a) == self.find_root(idx_b)
    }

    pub fn connect(&mut self, idx_a: usize, idx_b: usize) {
        let root_a = self.find_root(idx_a);
        let root_b = self.find_root(idx_b);
        if root_a != root_b {
            // 把a挂到b下
            self.group_idx[root_b] += self.group_idx[root_a];
            self.group_idx[root_a] = root_b as isize;
        }
    }
}

impl Clone for DisjointSet {
    fn clone(&self) -> Self {
        Self {
            group_idx: self.group_idx.clone(),
        }
    }
}
impl Debug for DisjointSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{:?}", self.group_idx)?;

        // Work on a temporary copy to avoid mutating self
        let mut tmp = self.clone();

        // root_idx -> member_idxs
        let mut groups: BTreeMap<usize, Vec<usize>> = BTreeMap::new();

        for idx in 0..tmp.group_idx.len() {
            let root_idx = tmp.find_root(idx);
            groups.entry(root_idx).or_default().push(idx);
        }

        writeln!(f, "Groups:")?;
        for (root_idx, member_idxs) in groups {
            writeln!(f, "    root@[{}], members: {:?}", root_idx, member_idxs)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test() {
        let mut d = DisjointSet::new(10);
        assert!(d.is_connected(1, 9) == false);
        d.connect(1, 9);
        assert!(d.is_connected(1, 9) == true);
        assert!(d.group_size(1) == 2);
        assert!(d.group_size(9) == 2);
        assert!(d.group_size(0) == 1);
        dbg!(&d);
        assert!(d.group_members(1) == vec![1, 9]);
        d.delete_group(1);
        assert!(d.group_members(1) == vec![1]);
        assert!(d.group_size(1) == 1);
        assert!(d.group_size(9) == 1);
        dbg!(&d);
    }
}
