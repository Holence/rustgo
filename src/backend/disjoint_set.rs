use crate::backend::{Array, Idx};
use std::collections::BTreeMap;
use std::fmt::Debug;

const NO_PARENT: usize = usize::MAX;

pub struct DisjointSet {
    /// 所属父级信息
    /// 非根节点: 存储父级下标
    /// 根节点: 存储 NO_PARENT
    /// 由于 NO_PARENT 选用了 usize 的上限值, 所以上限
    parent_idx: Array<Idx>,

    /// 成组大小
    /// 非根节点: 0
    /// 根节点: 组大小
    group_size: Array<usize>,
    // TODO 要不还是在这里存 group_members: Array<Option<Vec<Idx>>> 甚至可以把 group_size 省了
}

impl DisjointSet {
    pub fn new(capacity: usize) -> Self {
        // TODO 能否在new时通过判断capacity的大小范围, 创建不同的 DisjointSet<T>
        // TODO 大部分情况 都有 capacity <= u16::MAX, 可以不用 usize 存
        assert!(capacity < usize::MAX);
        DisjointSet {
            parent_idx: vec![NO_PARENT; capacity].into_boxed_slice(),
            group_size: vec![0; capacity].into_boxed_slice(),
        }
    }

    pub fn len(&self) -> usize {
        return self.parent_idx.len();
    }

    /// 寻找 idx 所属 group 的 group root
    ///
    /// 如果不存在 group, 则返回 None
    pub fn find_root(&mut self, idx: Idx) -> Option<Idx> {
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

    /// 强制做一次路径压缩, 一般情况无须手动调用
    pub fn run_path_compression(&mut self) {
        for idx in 0..self.len() {
            self.find_root(idx);
        }
    }

    /// 返回 idx 所属 group 的所有 member 的个数
    ///
    /// 如果不存在 group, 则返回 0
    pub fn group_size(&mut self, idx: Idx) -> usize {
        let root = self.find_root(idx);
        if let Some(root_idx) = root {
            return self.group_size[root_idx];
        } else {
            return 0;
        }
    }

    /// 返回 idx 所属 group 的所有 member
    ///
    /// 最好的情况时间复杂度 O(N), 所以最好不要频繁调用此函数
    pub fn group_members(&mut self, idx: Idx) -> Option<Vec<Idx>> {
        let root = self.find_root(idx);
        if root.is_none() {
            return None;
        }

        let root_idx = root.unwrap();
        let mut members: Vec<Idx> = vec![];
        for idx in 0..self.len() {
            if self.find_root(idx) == Some(root_idx) {
                members.push(idx);
            }
        }
        return Some(members);
    }

    /// 返回所有的 group root
    ///
    /// 时间复杂度 O(N)
    pub fn group_roots(&mut self) -> Vec<Idx> {
        let mut roots: Vec<Idx> = vec![];
        for idx in 0..self.len() {
            if self.group_size[idx] > 0 {
                roots.push(idx);
            }
        }
        return roots;
    }

    /// 删除 idx 所属 group 的所有元素
    ///
    /// 如果不存在 group, 则什么都不做
    pub fn delete_group(&mut self, idx: Idx) {
        let root = self.find_root(idx);
        if root.is_none() {
            return;
        }
        let root_idx = root.unwrap();

        for idx in self.group_members(idx).unwrap() {
            // 恢复初始值
            self.parent_idx[idx] = NO_PARENT;
        }
        self.group_size[root_idx] = 0;
    }

    pub fn is_connected(&mut self, idx_a: Idx, idx_b: Idx) -> bool {
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

    /// 添加一个元素, 创建孤立的 group
    ///
    /// 如果元素已经存在, 则什么都不做
    pub fn insert(&mut self, idx: Idx) {
        if !self.contains(idx) {
            self.group_size[idx] = 1;
        }
    }

    /// 是否存在元素
    pub fn contains(&self, idx: Idx) -> bool {
        self.parent_idx[idx] != NO_PARENT || self.group_size[idx] != 0
    }

    pub fn connect(&mut self, idx_a: Idx, idx_b: Idx) {
        self.insert(idx_a);
        self.insert(idx_b);

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
        let mut groups: BTreeMap<Idx, Vec<Idx>> = BTreeMap::new();

        for idx in 0..tmp.len() {
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
