use std::{
    collections::{HashSet, VecDeque},
    mem::replace,
};

use crate::backend::{
    Array, Coord, Idx, Stone,
    disjoint_set::DisjointSet,
    engine::group_info::{GroupInfo, GroupInfoArray},
};

mod group_info;
const MAX_STATES_RECORD: usize = 30;

#[derive(Debug)]
pub struct PlaceStoneResult {
    pub eaten_stones: Vec<Coord>, // 吃子坐标
}

pub type EngineResult = Result<PlaceStoneResult, &'static str>; // TODO err type

pub type Board = Array<Stone>;

pub struct Engine {
    size: usize,

    /// 棋盘所有坐标位置的一维存储 ( 2D_board[y][x] == board[y*size+x] ), 以左上角为原点, 向下为+y, 向右为+x
    ///
    /// board.len() == size * size
    board: Board,

    /// 棋子的分组信息
    ///
    /// 同色、连续的棋子在运行时使用 disjoint set 记录分组
    ///
    /// group root 所对应的下标 idx 在 self.group_info_array[idx] 中会记录额外的信息
    group_ds: DisjointSet,

    /// 棋子组的额外信息
    ///
    /// group_info_array.len() == size * size
    ///
    /// 1. 只在 idx == group root 时, 才有 group_info_array[idx] == Some(Box<GroupInfo>)
    /// 2. 其他情况, group_info_array[idx] == None
    group_info_array: GroupInfoArray,

    /// 为了判断全局同形而记录的历史状态
    ///
    /// 不需要记录全部的历史状态, 只需要记录最新的 ? 条即可 (TODO N劫循环的循环周期为多少)
    ///
    /// 新记录 push_front, 超出的 pop_back
    history_states: VecDeque<Board>, // TODO fixed size deque
}

impl Engine {
    pub fn new(size: usize) -> Self {
        Engine {
            size: size,
            board: vec![Stone::VOID; size * size].into_boxed_slice(),
            group_ds: DisjointSet::new(size * size),
            group_info_array: GroupInfoArray::new(size * size),
            history_states: VecDeque::with_capacity(MAX_STATES_RECORD),
        }
    }

    pub fn new_with_board(size: usize, board: Board) -> Self {
        debug_assert!(size * size == board.len());
        Engine {
            size,
            board,
            group_ds: DisjointSet::new(size * size),
            group_info_array: GroupInfoArray::new(size * size),
            history_states: VecDeque::with_capacity(MAX_STATES_RECORD),
        }
    }

    fn idx(&self, coord: Coord) -> Idx {
        debug_assert!(coord.y < self.size);
        debug_assert!(coord.x < self.size);
        return coord.y * self.size + coord.x;
    }

    fn neighbors(&self, idx: Idx) -> Vec<Idx> {
        let mut v: Vec<Idx> = Vec::with_capacity(4); // TODO array vec on stack
        let y = idx / self.size;
        let x = idx % self.size;
        if x > 0 {
            v.push((x - 1) + self.size * (y));
        }
        if x < self.size - 1 {
            v.push((x + 1) + self.size * (y));
        }
        if y > 0 {
            v.push((x) + self.size * (y - 1));
        }
        if y < self.size - 1 {
            v.push((x) + self.size * (y + 1));
        }
        return v;
    }

    fn neighbor_groups(&mut self, idx: Idx) -> Vec<Idx> {
        let mut v: Vec<Idx> = Vec::new(); // TODO array vec on stack
        let y = idx / self.size;
        let x = idx % self.size;
        let mut neigbor_idx: Idx;
        if x > 0 {
            neigbor_idx = (x - 1) + self.size * (y);
            if self.have_stone(neigbor_idx) {
                push_if_not_exist(&mut v, self.group_ds.find_root(neigbor_idx));
            }
        }
        if x < self.size - 1 {
            neigbor_idx = (x + 1) + self.size * (y);
            if self.have_stone(neigbor_idx) {
                push_if_not_exist(&mut v, self.group_ds.find_root(neigbor_idx));
            }
        }
        if y > 0 {
            neigbor_idx = (x) + self.size * (y - 1);
            if self.have_stone(neigbor_idx) {
                push_if_not_exist(&mut v, self.group_ds.find_root(neigbor_idx));
            }
        }
        if y < self.size - 1 {
            neigbor_idx = (x) + self.size * (y + 1);
            if self.have_stone(neigbor_idx) {
                push_if_not_exist(&mut v, self.group_ds.find_root(neigbor_idx));
            }
        }
        return v;
    }

    fn have_stone(&self, idx: Idx) -> bool {
        self.board[idx] != Stone::VOID
    }

    fn calc_qi(&self, members: &Vec<Idx>) -> usize {
        let mut voids: HashSet<Idx> = HashSet::with_capacity(members.len());
        for &idx in members {
            for neighbor in self.neighbors(idx) {
                if !self.have_stone(neighbor) {
                    voids.insert(neighbor);
                }
            }
        }
        return voids.len();
    }

    #[cfg(debug_assertions)]
    fn verbose_check(&self) {
        for idx in 0..self.board.len() {
            if self.have_stone(idx) {
                let mut tmp = self.group_ds.clone();
                let root_idx = tmp.find_root(idx);

                let group_info = self.group_info_array.get(root_idx);

                // check group members
                let b: Vec<usize> = group_info.members.clone();
                let b: HashSet<usize> = b.into_iter().collect();
                let a: Vec<usize> = tmp.group_members(idx);
                let a: HashSet<usize> = a.into_iter().collect();
                debug_assert!(HashSet::difference(&a, &b).count() == 0);

                // check 气
                debug_assert!(group_info.qi != 0);
                debug_assert!(self.calc_qi(&group_info.members) == group_info.qi);

                // TODO check 连接性
            }
        }
    }

    pub fn place_stone(&mut self, coord: Coord, stone: Stone) -> EngineResult {
        debug_assert!(stone != Stone::VOID);

        #[cfg(debug_assertions)]
        self.verbose_check();

        let cur_idx = self.idx(coord);
        debug_assert!(cur_idx < self.board.len());

        // 1. 禁止下到已有的棋子上
        if self.have_stone(cur_idx) {
            return Err("禁止下到已有的棋子上");
        }

        // 2. 计算落子位置的"气"为 `cur_qi`
        // 3. 找出落子周围的"非己方组"与"己方组"
        //    其中"提子组"定义为: "非己方组" 且 "气"为1
        let mut cur_qi: usize = 0;
        // TODO array vec on stack
        let mut ally_groups: Vec<Idx> = Vec::with_capacity(4); // "己方组"
        let mut opponent_groups: Vec<Idx> = Vec::with_capacity(4); // "非己方组" (不包含"提子组")
        let mut eaten_groups: Vec<Idx> = Vec::with_capacity(4); // "提子组"
        for neighbor_idx in self.neighbors(cur_idx) {
            let neighbor_stone = self.board[neighbor_idx];
            if neighbor_stone == Stone::VOID {
                cur_qi += 1;
            } else {
                let root_idx = self.group_ds.find_root(neighbor_idx);
                let group_info = self.group_info_array.get(root_idx);
                debug_assert!(group_info.members.len() > 0);
                debug_assert!(group_info.qi > 0);
                if neighbor_stone == stone {
                    push_if_not_exist(&mut ally_groups, root_idx);
                } else {
                    if group_info.qi == 1 {
                        push_if_not_exist(&mut eaten_groups, root_idx);
                    } else {
                        push_if_not_exist(&mut opponent_groups, root_idx);
                    }
                }
            }
        }

        // 4. 禁止使己方气尽: 如果没有"提子组", 且`cur_qi==0`且所有"己方组"的"气"都是1, 则判定为自杀
        if eaten_groups.len() == 0 {
            if cur_qi == 0 {
                let mut flag = false;
                for &root_idx in &ally_groups {
                    if self.group_info_array.get(root_idx).qi != 1 {
                        flag = true;
                        break;
                    }
                }
                if flag == false {
                    return Err("禁止使己方气尽");
                }
            }
        }

        // 5. 禁止全局同形: "棋盘经过落子+提子的变化" 与 list[历史记录] 比较, 不可以相同
        // TODO 存储压缩
        let mut new_board = self.board.clone();
        new_board[cur_idx] = stone;
        for &root_idx in &eaten_groups {
            for &idx in &(self.group_info_array.get(root_idx).members) {
                new_board[idx] = Stone::VOID;
            }
        }
        for board in &self.history_states {
            if *board == new_board {
                return Err("禁止全局同形");
            }
        }

        // 6. 之后便允许落子
        self.board[cur_idx] = stone;

        // 6.1 如果有"己方组", 则将落子与"己方组"merge, group root可能会更新, 在group root中更新"气"和members
        //     (此时气可能为0, 要等到提子后才还会被接着更新)
        if ally_groups.len() == 0 {
            // 自己成组
            self.group_info_array[cur_idx] = Some(GroupInfo::new(cur_qi, vec![cur_idx]));
        } else {
            // TODO 很难归纳出通过简单加加减减merge group气的算法, 因为还需要考虑公气
            // 这里直接粗暴merge, 再重新计算整个group的气
            let mut members: Vec<Idx> = vec![cur_idx];
            for root_idx in ally_groups {
                self.group_ds.connect(cur_idx, root_idx);
                let group_info = self.group_info_array[root_idx].take().unwrap(); // take out, and free
                members.extend(group_info.members);
            }

            let root_idx = self.group_ds.find_root(cur_idx);
            let qi = self.calc_qi(&members);

            if self.group_info_array[root_idx].is_none() {
                self.group_info_array[root_idx] = Some(GroupInfo::new(qi, members));
            } else {
                let group_info = self.group_info_array.get_mut(root_idx);
                group_info.qi = qi;
                let _ = replace(&mut group_info.members, members);
            }
        }

        // 6.2 如果有"非己方组"且不是"提子组", 则用落子更新"气"
        for root_idx in opponent_groups {
            self.group_info_array.get_mut(root_idx).qi -= 1;
        }

        // 6.3 如果有"提子组", 则把所有"提子组"的members统计为一个list, 棋盘上这些坐标置空, 遍历list, 对于每个member遗址, 更新遗址周围的组的"气"
        //     (这里之所以要先把所有"提子组"merge为list再遍历, 而不是对每个"提子组"依次遍历, 是因为考虑到N色棋的提子情况, 一次落子可能提走几种颜色的"非己方组")
        // TODO more test
        // TODO test n色棋
        let mut eaten_stones: Vec<Idx> = vec![];
        for root_idx in eaten_groups {
            let group = self.group_info_array[root_idx].take().unwrap(); // take out, and free
            eaten_stones.extend(group.members);
            self.group_ds.delete_group(root_idx);
        }
        for &idx in &eaten_stones {
            self.board[idx] = Stone::VOID;
        }
        for &idx in &eaten_stones {
            for root_idx in self.neighbor_groups(idx) {
                self.group_info_array.get_mut(root_idx).qi += 1;
            }
        }

        debug_assert!(new_board == self.board);
        if self.history_states.len() == MAX_STATES_RECORD {
            self.history_states.pop_back();
        }
        self.history_states.push_front(new_board);

        Ok(PlaceStoneResult {
            eaten_stones: eaten_stones
                .iter()
                .map(|idx| Coord::new(idx % self.size, idx / self.size))
                .collect(),
        })
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn board(&self) -> &[Stone] {
        &self.board
    }

    pub fn board_string(&self) -> String {
        let mut s = String::with_capacity(self.size * self.size * 2);
        let mut idx = 0;
        for _ in 0..self.size {
            for _ in 0..self.size {
                let ch = self.board[idx].as_char();
                s.push(ch);
                idx += 1;
            }
            s.push('\n');
        }
        return s;
    }
}

fn push_if_not_exist(v: &mut Vec<usize>, x: usize) {
    if !v.contains(&x) {
        v.push(x);
    }
}
