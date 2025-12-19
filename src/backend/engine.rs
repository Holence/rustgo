use std::{collections::HashSet, mem::replace};

use crate::backend::{Array, Coord, Stone, disjoint_set::DisjointSet};

pub struct PlaceStoneResult {
    pub eaten: Vec<Coord>, // 吃子坐标
}

pub type EngineResult = Result<PlaceStoneResult, &'static str>;

pub type Board = Array<Stone>;
type Idx = usize;

// TODO more assert on this info
#[derive(Clone, Debug)]
struct GroupInfo {
    stone: Stone,      // group的棋子颜色 TODO 是否无用?
    qi: usize,         // group的气
    members: Vec<Idx>, // group的所有棋子
}

impl GroupInfo {
    fn new(stone: Stone, qi: usize, members: Vec<Idx>) -> Self {
        Self { stone, qi, members }
    }
}

pub struct Engine {
    size: usize,

    /// 棋盘所有坐标位置的一维存储 ( 2D_board[y][x] == board[y*size+x] ), 以左上角为原点, 向下为+y, 向右为+x
    ///
    /// board.len() == size * size
    board: Board,

    /// 记录棋子组, 同色、连续的棋子在运行时用 disjoint set 来分组记录
    group_ds: DisjointSet,

    /// 棋子组的额外信息
    ///
    /// group_info.len() == size * size
    ///
    /// 1. 只在 idx == group root 时, 才有 group_info[idx] == Some(Box<GroupInfo>)
    /// 2. 其他情况, group_info[idx] == None
    group_info: Array<Option<Box<GroupInfo>>>,
}

impl Engine {
    pub fn new(size: usize) -> Self {
        Engine {
            size: size,
            board: vec![Stone::Void; size * size].into_boxed_slice(),
            group_ds: DisjointSet::new(size * size),
            group_info: vec![None; size * size].into_boxed_slice(),
        }
    }

    pub fn new_with_board(size: usize, board: Board) -> Self {
        debug_assert!(size * size == board.len());
        Engine {
            size,
            board,
            group_ds: DisjointSet::new(size * size),
            group_info: vec![None; size * size].into_boxed_slice(),
        }
    }

    fn idx(&self, coord: Coord) -> Idx {
        debug_assert!(coord.y < self.size);
        debug_assert!(coord.x < self.size);
        return coord.y * self.size + coord.x;
    }

    fn neighbors(&self, idx: Idx) -> Vec<Idx> {
        let mut v: Vec<Idx> = Vec::new();
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

    fn have_stone(&self, idx: Idx) -> bool {
        self.board[idx] != Stone::Void
    }

    fn calc_qi(&self, members: &Vec<Idx>) -> usize {
        let mut voids: HashSet<Idx> = HashSet::new();
        for member in members {
            for neighbor in self.neighbors(*member) {
                if !self.have_stone(neighbor) {
                    voids.insert(neighbor);
                }
            }
        }
        return voids.len();
    }

    pub fn place_stone(&mut self, coord: Coord, stone: Stone) -> EngineResult {
        debug_assert!(stone != Stone::Void);

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
        let mut self_groups: Vec<Idx> = Vec::with_capacity(4); // TODO array vec on stack
        let mut eaten_groups: Vec<Idx> = Vec::with_capacity(4);
        let mut other_groups: Vec<Idx> = Vec::with_capacity(4);
        for neighbor in self.neighbors(cur_idx) {
            let neighbor_stone = self.board[neighbor];
            if neighbor_stone == Stone::Void {
                cur_qi += 1;
            } else {
                let root_idx = self.group_ds.find_root(neighbor);
                let group_info = self.group_info[root_idx].as_ref().unwrap();
                debug_assert!(group_info.members.len() > 0);
                debug_assert!(group_info.qi > 0);
                if neighbor_stone == stone {
                    debug_assert!(group_info.stone == stone);
                    push_if_not_exist(&mut self_groups, root_idx);
                } else {
                    debug_assert!(group_info.stone != stone);
                    if group_info.qi == 1 {
                        push_if_not_exist(&mut eaten_groups, root_idx);
                    } else {
                        push_if_not_exist(&mut other_groups, root_idx);
                    }
                }
            }
        }

        // 4. 禁止自杀: 如果没有"提子组", 且`cur_qi==0`且所有"己方组"的"气"都是1, 则判定为自杀
        if eaten_groups.len() == 0 {
            if cur_qi == 0 {
                let mut flag = false;
                for &root_idx in &self_groups {
                    if self.group_info[root_idx].as_ref().unwrap().qi != 1 {
                        flag = true;
                        break;
                    }
                }
                if flag == false {
                    return Err("禁止自杀");
                }
            }
        }

        // 5. 禁止全局同形: "棋盘经过落子+提子的变化" 与 list[历史记录] 比较, 不可以相同
        // TODO

        // 6. 之后便允许落子
        self.board[cur_idx] = stone;

        // 6.1 如果有"己方组", 则将落子与"己方组"merge, group root可能会更新, 在group root中更新"气"和members
        //     (此时气可能为0, 要等到提子后才还会被接着更新)
        if self_groups.len() == 0 {
            // 自己成组
            self.group_info[cur_idx] = Some(Box::new(GroupInfo::new(stone, cur_qi, vec![cur_idx])));
        } else {
            // TODO 很难归纳出通过简单加加减减merge group气的算法，因为还需要考虑公气
            // 这里直接粗暴merge, 再重新计算整个group的气
            let mut members: Vec<Idx> = vec![cur_idx];
            for root_idx in self_groups {
                self.group_ds.connect(cur_idx, root_idx);
                let group = self.group_info[root_idx].take().unwrap(); // take out, and free
                members.extend(group.members);
            }

            let root_idx = self.group_ds.find_root(cur_idx);

            let qi = self.calc_qi(&members);
            let option = self.group_info[root_idx].as_mut();
            match option {
                Some(group) => {
                    group.qi = qi;
                    let _ = replace(&mut group.members, members);
                }
                None => {
                    self.group_info[root_idx] = Some(Box::new(GroupInfo::new(stone, qi, members)));
                }
            }
        }

        // 6.2 如果有"非己方组"且不是"提子组", 则用落子更新"气"
        for root_idx in other_groups {
            self.group_info[root_idx].as_mut().unwrap().qi -= 1;
        }

        // 6.3 如果有"提子组", 则把所有"提子组"的members统计为一个list, 棋盘上这些坐标置空, 遍历list, 对于每个member遗址, 更新遗址周围的组的"气"
        //     (这里之所以要先把所有"提子组"merge为list再遍历, 而不是对每个"提子组"依次遍历, 是因为考虑到N色棋的提子情况, 一次落子可能提走几种颜色的"非己方组")

        Ok(PlaceStoneResult { eaten: vec![] })
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
