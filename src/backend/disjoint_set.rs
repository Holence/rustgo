use crate::backend::{Array, Idx};
use std::collections::BTreeMap;
use std::fmt::Debug;

const NO_PARENT: usize = usize::MAX;

#[derive(Clone)]
pub struct DisjointSet {
    /// 所属父级信息
    /// 非根节点: 存储父级下标
    /// 根节点: 存储 NO_PARENT
    /// 由于 NO_PARENT 选用了 usize 的上限值, 所以上限
    parent_idx: Array<Idx>,

    /// 组成员
    /// 非根节点: None
    /// 根节点: Vec of members
    group_members: Array<Option<Vec<Idx>>>,
}

impl DisjointSet {
    /// 创建容量为 `capacity` 的并查集 (目前的实现是不可扩容的)
    pub fn new(capacity: usize) -> Self {
        // TODO 能否在new时通过判断capacity的大小范围, 创建不同的 DisjointSet<T>
        // TODO 大部分情况 都有 capacity <= u16::MAX, 可以不用 usize 存
        assert!(capacity < usize::MAX);
        DisjointSet {
            parent_idx: vec![NO_PARENT; capacity].into_boxed_slice(),
            group_members: vec![None; capacity].into_boxed_slice(),
        }
    }

    /// 最大能承载多少元素
    pub fn capacity(&self) -> usize {
        return self.parent_idx.len();
    }

    /// 已经存储了多少元素
    pub fn len(&self) -> usize {
        self.group_members.iter().filter(|x| x.is_some()).count()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// 寻找 idx 所属 group 的 group root
    ///
    /// 如果不存在 group, 则返回 None
    pub fn find_root(&mut self, idx: Idx) -> Option<Idx> {
        let parent_idx = self.parent_idx[idx];
        if parent_idx == NO_PARENT {
            if self.group_members[idx].is_none() {
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
        for idx in 0..self.capacity() {
            self.find_root(idx);
        }
    }

    /// 返回 idx 所属 group 的所有 member 的个数
    ///
    /// 如果不存在 group, 则返回 0
    pub fn group_size(&mut self, idx: Idx) -> usize {
        let root = self.find_root(idx);
        if let Some(root_idx) = root {
            return self.group_members[root_idx].as_ref().unwrap().len();
        } else {
            return 0;
        }
    }

    /// 返回 idx 所属 group 的所有 member (保证升序)
    pub fn group_members(&mut self, idx: Idx) -> Option<&Vec<Idx>> {
        let root_idx = self.find_root(idx)?;
        let members = self.group_members[root_idx].as_mut().unwrap();
        members.sort_unstable(); // 排序不放在 connect 里, 因为 connect 调用的更频繁
        return Some(members);
    }

    /// 返回所有的 group root
    ///
    /// 时间复杂度 O(N)
    pub fn group_roots(&mut self) -> Vec<Idx> {
        let mut roots: Vec<Idx> = vec![];
        for idx in 0..self.capacity() {
            if self.group_members[idx].is_some() {
                roots.push(idx);
            }
        }
        return roots;
    }

    /// 删除 idx 所属 group 的所有 members, 并返回该 group 的 members (保证升序)
    ///
    /// # Panic
    ///
    /// 如果不存在 group, 则 Panic
    pub fn delete_group(&mut self, idx: Idx) -> Vec<Idx> {
        let root = self.find_root(idx);
        if root.is_none() {
            panic!("idx should belong to a group!");
        }
        let root_idx = root.unwrap();

        let mut members = self.group_members[root_idx].take().unwrap(); // take out, leave as None
        for &idx in &members {
            // 恢复初始值
            self.parent_idx[idx] = NO_PARENT;
        }
        members.sort_unstable(); // 排序不放在 connect 里, 因为 connect 调用的更频繁
        return members;
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
            self.group_members[idx] = Some(vec![idx]);
        }
    }

    /// 是否存在元素
    pub fn contains(&self, idx: Idx) -> bool {
        self.parent_idx[idx] != NO_PARENT || self.group_members[idx].is_some()
    }

    pub fn connect(&mut self, idx_a: Idx, idx_b: Idx) {
        self.insert(idx_a);
        self.insert(idx_b);

        let root_a = self.find_root(idx_a).unwrap();
        let root_b = self.find_root(idx_b).unwrap();
        if root_a != root_b {
            // 把a挂到b下
            let mut members_a = self.group_members[root_a].take().unwrap(); // take out, leave as None

            self.group_members[root_b]
                .as_mut()
                .unwrap()
                .append(&mut members_a);

            self.parent_idx[root_a] = root_b;
        }
    }
}

impl Debug for DisjointSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{:?}", self.parent_idx)?;
        writeln!(f, "{:?}", self.group_members)?;

        // Work on a temporary copy to avoid mutating self
        let mut tmp = self.clone();

        // root_idx -> member_idxs
        let mut groups: BTreeMap<Idx, Vec<Idx>> = BTreeMap::new();

        for idx in 0..tmp.capacity() {
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
