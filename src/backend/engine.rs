use crate::backend::{Array, Coord, Stone, disjoint_set::DisjointSet};

pub struct PlaceStoneResult {
    pub eaten: Vec<Coord>, // 吃子坐标
}

pub type EngineResult = Result<PlaceStoneResult, &'static str>;

pub type Board = Array<Stone>;

#[derive(Clone)]
struct GroupInfo {
    qi: usize,    // group的气
    color: Stone, // group的颜色
}

pub struct Engine {
    size: usize,

    /// 棋盘所有坐标位置的一维存储 ( 2D_board[y][x] == board[y*size+x] ), 以左上角为原点, 向下为+y, 向右为+x
    ///
    /// board.len() == size * size
    board: Board,

    /// 同色、连续的棋子在运行时用 disjoint set 来分组记录
    ///
    /// group_idx.len() == size * size
    ///
    /// 1. 初始时 group_idx[idx] == idx, 表示 board[idx] 所对应的位置不存在棋子
    /// 2. 在 board[idx] 处放置了棋子后, group_idx[idx] == -1, 表示 board[idx] 所对应的位置成为了棋子组
    /// 3. 在 board[idx+1] 处放置了棋子后, group_idx[idx+1] == idx, group_idx[idx] == -2, 表示 board[idx+1] 的棋子归属于 board[idx] 统帅, board[idx] 统帅着 2 个棋子
    /// 4. 若 group_idx[idx] == -num, 则表示为 board[idx] 所对应的位置是某个棋子组的首领, 它统帅着 num 个棋子
    stone_disjoint_set: DisjointSet,

    /// 棋子组的额外信息
    ///
    /// group_info.len() == size * size
    ///
    /// 1. 只在 group_idx[idx] == -num 时, 才有 group_info[idx] == Some(Box<GroupInfo>)
    /// 2. 其他情况, group_info[idx] == None
    group_info: Array<Option<Box<GroupInfo>>>,
}

impl Engine {
    pub fn new(size: usize) -> Self {
        Engine {
            size: size,
            board: vec![Stone::Void; size * size].into_boxed_slice(),
            stone_disjoint_set: DisjointSet::new(size * size),
            group_info: vec![None; size * size].into_boxed_slice(),
        }
    }

    pub fn with_board(size: usize, board: Board) -> Self {
        debug_assert!(size * size == board.len());
        Engine {
            size,
            board,
            stone_disjoint_set: DisjointSet::new(size * size),
            group_info: vec![None; size * size].into_boxed_slice(),
        }
    }

    fn idx(&self, coord: Coord) -> usize {
        debug_assert!(coord.y < self.size);
        debug_assert!(coord.x < self.size);
        return coord.y * self.size + coord.x;
    }

    fn neighbor_coords(&self, coord: Coord) -> std::vec::IntoIter<Coord> {
        let mut v: Vec<Coord> = Vec::new();
        if coord.x > 0 {
            v.push(Coord::new(coord.x - 1, coord.y));
        }
        if coord.x < self.size - 1 {
            v.push(Coord::new(coord.x + 1, coord.y));
        }
        if coord.y > 0 {
            v.push(Coord::new(coord.x, coord.y - 1));
        }
        if coord.y < self.size - 1 {
            v.push(Coord::new(coord.x, coord.y + 1));
        }
        return v.into_iter();
    }

    fn have_stone(&self, coord: Coord) -> bool {
        self.board[self.idx(coord)] != Stone::Void
    }

    pub fn place_stone(&mut self, coord: Coord, stone: Stone) -> EngineResult {
        debug_assert!(stone != Stone::Void);

        let idx = self.idx(coord);
        debug_assert!(idx < self.board.len());

        // 1. 禁止下到已有的棋子上
        if self.have_stone(coord) {
            return Err("禁止下到已有的棋子上");
        }

        // 2. 计算落子位置的"气"为 `cur_qi`
        let mut cur_qi: usize = 0;
        for neighbor_coord in self.neighbor_coords(coord) {
            if !self.have_stone(neighbor_coord) {
                cur_qi += 1;
            }
        }

        // 3. 找出落子周围的"非己方组"与"己方组"
        //    其中"提子组"定义为: "非己方组" 且 "气"为1

        // 4. 禁止自杀: 如果没有"提子组", 且`cur_qi==0`且所有"己方组"的"气"都是1, 则判定为自杀

        // 5. 禁止全局同形: "棋盘经过落子+提子的变化" 与 list[历史记录] 比较, 不可以相同

        // 6. 之后便允许落子

        // 6.1 如果有"己方组", 则将落子与"己方组"merge, group root可能会更新, 在group root中更新"气"和members
        //     (此时气可能为0, 要等到提子后才还会被接着更新)

        // 6.2 如果有"非己方组"且不是"提子组", 则用落子更新"气"

        // 6.3 如果有"提子组", 则把所有"提子组"的members统计为一个list, 棋盘上这些坐标置空, 遍历list, 对于每个member遗址, 更新遗址周围的组的"气"
        //     (这里之所以要先把所有"提子组"merge为list再遍历, 而不是对每个"提子组"依次遍历, 是因为考虑到N色棋的提子情况, 一次落子可能提走几种颜色的"非己方组")

        self.board[idx] = stone;
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
