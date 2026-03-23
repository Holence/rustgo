use std::collections::BTreeMap;
use std::fmt::Debug;

use crate::Array;

/// Trait for compact index storage
pub trait IdxTrait: Copy + Eq + Debug {
    fn from_usize(x: usize) -> Self;
    fn to_usize(self) -> usize;
    fn max_value() -> usize;
}

macro_rules! impl_idx {
    ($t:ty) => {
        impl IdxTrait for $t {
            #[inline]
            fn from_usize(x: usize) -> Self {
                x as $t
            }

            #[inline]
            fn to_usize(self) -> usize {
                self as usize
            }

            #[inline]
            fn max_value() -> usize {
                <$t>::MAX as usize
            }
        }
    };
}

impl_idx!(u8);
impl_idx!(u16);
impl_idx!(u32);
impl_idx!(u64);
impl_idx!(usize);

#[derive(Clone)]
pub struct DisjointSet<T: IdxTrait> {
    /// 所属父级信息
    /// 非根节点: 父级下标
    /// 根节点: None
    parent_idx: Array<Option<T>>,

    /// 组成员
    /// 非根节点: None
    /// 根节点: Vec of members
    group_members: Array<Option<Vec<usize>>>,
}

impl<T: IdxTrait> DisjointSet<T> {
    /// 创建容量为 `capacity` 的并查集 (目前的实现是不可扩容的)
    pub fn new(capacity: usize) -> Self {
        assert!(
            capacity <= T::max_value(),
            "capacity {} exceeds max storable index {}",
            capacity,
            T::max_value()
        );

        DisjointSet {
            parent_idx: vec![None; capacity].into_boxed_slice(),
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
    pub fn find_root(&mut self, idx: usize) -> Option<usize> {
        let parent_idx = self.parent_idx[idx];
        if let Some(parent_idx) = parent_idx {
            let root = self.find_root(parent_idx.to_usize());
            if let Some(root_idx) = root {
                self.parent_idx[idx] = Some(T::from_usize(root_idx)); // 路径压缩
            }
            return root;
        } else {
            if self.group_members[idx].is_none() {
                return None;
            } else {
                return Some(idx);
            }
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
    pub fn group_size(&mut self, idx: usize) -> usize {
        let root = self.find_root(idx);
        if let Some(root_idx) = root {
            return self.group_members[root_idx].as_ref().unwrap().len();
        } else {
            return 0;
        }
    }

    /// 返回 idx 所属 group 的所有 member (保证升序)
    pub fn group_members(&mut self, idx: usize) -> Option<&Vec<usize>> {
        let root_idx = self.find_root(idx)?;
        let members = self.group_members[root_idx].as_mut().unwrap();
        members.sort_unstable(); // 排序不放在 connect 里, 因为 connect 调用的更频繁
        return Some(members);
    }

    /// 返回所有的 group root
    ///
    /// 时间复杂度 O(N)
    pub fn group_roots(&self) -> Vec<usize> {
        let mut roots = vec![];
        for idx in 0..self.capacity() {
            if self.group_members[idx].is_some() {
                roots.push(idx);
            }
        }
        return roots;
    }

    /// 删除 idx 所属 group 的所有 members, 并返回该 group 的 Some(members) (保证升序)
    ///
    /// 如果不存在 group, 则 None
    pub fn delete_group(&mut self, idx: usize) -> Option<Vec<usize>> {
        let root_idx = self.find_root(idx)?;

        let mut members = self.group_members[root_idx].take().unwrap(); // take out, leave as None
        for &idx in &members {
            // 恢复初始值
            self.parent_idx[idx] = None;
        }
        members.sort_unstable(); // 排序不放在 connect 里, 因为 connect 调用的更频繁
        return Some(members);
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

    /// 添加一个元素, 创建孤立的 group
    ///
    /// 如果元素已经存在, 则什么都不做
    pub fn insert(&mut self, idx: usize) {
        if !self.contains(idx) {
            self.group_members[idx] = Some(vec![idx]);
        }
    }

    /// 是否存在元素
    pub fn contains(&self, idx: usize) -> bool {
        self.parent_idx[idx].is_some() || self.group_members[idx].is_some()
    }

    pub fn connect(&mut self, idx_a: usize, idx_b: usize) {
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

            self.parent_idx[root_a] = Some(T::from_usize(root_b));
        }
    }
}

impl<T: IdxTrait> Debug for DisjointSet<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{:?}", self.parent_idx)?;
        writeln!(f, "{:?}", self.group_members)?;

        // Work on a temporary copy to avoid mutating self
        let mut tmp = self.clone();

        // root_idx -> member_idxs
        let mut groups: BTreeMap<usize, Vec<usize>> = BTreeMap::new();

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
