use crate::{backend::Coord, backend::Stone};

pub struct PlaceStoneResult {
    pub eaten: Vec<Coord>, // 吃子坐标
}

pub type EngineResult = Result<PlaceStoneResult, &'static str>;

pub type Board = Box<[Stone]>; // 以左上角为原点，向下为+y，向右为+x

pub struct Engine {
    size: usize,
    board: Board,
}

impl Engine {
    pub fn new(size: usize) -> Self {
        Engine {
            size: size,
            board: vec![Stone::Void; size * size].into_boxed_slice(),
        }
    }

    pub fn with_board(size: usize, board: Board) -> Self {
        debug_assert!(size * size == board.len());
        Engine { size, board }
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

        // 2. 禁止全局同形 TODO

        // 3. 计算吃子 TODO
        let mut coord_qi: usize = 0;
        let mut eaten: Vec<Coord> = vec![];
        for neighbor_coord in self.neighbor_coords(coord) {
            if !self.have_stone(neighbor_coord) {
                coord_qi += 1;
            }
            // 检测周围group的气是否被当前落子更新为0, 更新为0的group即为被吃的子
        }

        // 如果没有吃子发生，且本坐标的气为0，则为自杀行为
        if coord_qi == 0 && eaten.len() == 0 {
            return Err("禁止使己方气尽");
        }

        self.board[idx] = stone;
        Ok(PlaceStoneResult { eaten: eaten })
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
