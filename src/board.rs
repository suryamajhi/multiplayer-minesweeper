use crate::board::CellType::{Flagged, Untouched};
use rand::{random, Rng};
use std::fmt::{Display, Formatter};

enum CellType {
    NoNeighborBombs,
    Untouched,
    Flagged,
}

impl CellType {
    fn value(&self) -> char {
        match self {
            CellType::NoNeighborBombs => ' ',
            CellType::Untouched => '-',
            CellType::Flagged => 'F',
        }
    }
}

pub struct Board {
    player_count: usize,
    size_x: usize,
    size_y: usize,
    dirs: [[i8; 2]; 8],
    board: Vec<Vec<char>>,
    bomb: Vec<Vec<bool>>,
}

impl Board {
    pub fn new(size_x: usize, size_y: usize) -> Self {
        let bomb_probability = 0.3f32;
        let mut board = vec![vec!['#'; size_y]; size_x];
        let mut bomb = vec![vec![false; size_y]; size_x];
        let mut rng = rand::thread_rng();

        for i in 0..size_x {
            for j in 0..size_y {
                board[i][j] = CellType::Untouched.value();
                if rng.gen_range(0.0..1.0) <= bomb_probability {
                    bomb[i][j] = true;
                }
            }
        }

        Board {
            player_count: 0,
            size_x,
            size_y,
            dirs: [
                [-1, -1],
                [-1, 0],
                [-1, 1],
                [0, 1],
                [0, -1],
                [1, 0],
                [1, -1],
                [1, 1],
            ],
            board,
            bomb,
        }
    }

    /// Assert the correctness of the board and it's representation
    fn check_rep(&self) {
        assert_eq!(self.board.len(), self.size_x);
        assert_eq!(self.board.get(0).unwrap().len(), self.size_y);
        assert_eq!(self.bomb.len(), self.size_x);
        assert_eq!(self.bomb.get(0).unwrap().len(), self.size_y);
    }

    pub fn get_size_x(&self) -> usize {
        self.size_x
    }

    pub fn get_size_y(&self) -> usize {
        self.size_y
    }

    pub fn add_player(&mut self) -> usize {
        self.player_count = self.player_count + 1;
        self.player_count
    }

    pub fn remove_player(&mut self) {
        self.player_count = self.player_count - 1;
    }

    fn is_valid_coordinate(&self, x: i32, y: i32) -> bool {
        x >= 0 && x < self.size_x as i32 && y >= 0 && y < self.size_y as i32
    }

    fn get_adjacent_bombs(&self, x: i32, y: i32) -> usize {
        let mut bomb_count = 0;
        for dir in self.dirs {
            let new_x = x + dir[0] as i32;
            let new_y = y + dir[1] as i32;
            if self.is_valid_coordinate(new_x, new_y) && self.bomb[new_x as usize][new_y as usize] {
                bomb_count += 1;
            }
        }
        bomb_count
    }

    pub fn flag(&mut self, x: i32, y: i32) -> String {
        if self.is_valid_coordinate(x, y)
            && self.board[x as usize][y as usize] == CellType::Untouched.value()
        {
            self.board[x as usize][y as usize] = Flagged.value();
        }
        format!("{}", self)
    }

    pub fn deflag(&mut self, x: i32, y: i32) -> String {
        if self.is_valid_coordinate(x, y) && self.board[x as usize][y as usize] == Flagged.value() {
            self.board[x as usize][y as usize] = Untouched.value();
        }
        format!("{}", self)
    }

    pub fn dig(&mut self, x: i32, y: i32) -> String {
        "".into()
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut result = String::new();
        for i in 0..self.board.len() {
            for j in 0..self.board.len() {
                result.push(self.board[i][j]);
                if j != self.board[i].len() - 1 {
                    result.push(' ');
                }
            }
            result.push('\n');
        }
        result.pop();
        write!(f, "{}", result)
    }
}
