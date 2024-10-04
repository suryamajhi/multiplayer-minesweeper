use std::collections::{HashSet, VecDeque};
use std::fmt::{Display, Formatter};

use rand::Rng;

use crate::board::CellType::{Flagged, NoNeighborBombs, Untouched};

enum CellType {
    NoNeighborBombs,
    Untouched,
    Flagged,
}

impl CellType {
    fn value(&self) -> char {
        match self {
            NoNeighborBombs => ' ',
            Untouched => '-',
            Flagged => 'F',
        }
    }
}

pub struct Board {
    player_count: usize,
    size_x: usize,
    size_y: usize,
    dirs: [[i32; 2]; 8],
    board: Vec<Vec<char>>,
    bomb: Vec<Vec<bool>>,
}

impl Board {
    pub fn new(size_x: usize, size_y: usize) -> Self {
        let bomb_probability = 0.2f32;
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

        for i in 0..self.board.len() {
            for j in 0..self.board[i].len() {
                assert!(
                    self.board[i][j] == Flagged.value()
                        || self.board[i][j] == Untouched.value()
                        || (self.board[i][j] >= '1' && self.board[i][j] <= '8'
                            || self.board[i][j] == NoNeighborBombs.value())
                );
                if self.board[i][j] != Flagged.value() && self.board[i][j] != Untouched.value() {
                    let surrounding_bombs = self.get_adjacent_bombs(i, j);
                    if surrounding_bombs == 0 {
                        assert_eq!(self.board[i][j], NoNeighborBombs.value());
                    } else {
                        assert_eq!(
                            self.board[i][j],
                            char::from_digit(surrounding_bombs as u32, 10).unwrap()
                        );
                    }
                }
            }
        }
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

    fn get_adjacent_bombs(&self, x: usize, y: usize) -> usize {
        let mut bomb_count = 0;
        for dir in self.dirs {
            let new_x = x as i32 + dir[0];
            let new_y = y as i32 + dir[1];
            if self.is_valid_coordinate(new_x, new_y) && self.bomb[new_x as usize][new_y as usize] {
                bomb_count += 1;
            }
        }
        bomb_count
    }

    /// Flag the coordinate of the board
    ///
    /// # Arguments
    ///
    /// * `x`: x-coordinate
    /// * `y`: y-coordinate
    ///
    /// returns: the board representation
    pub fn flag(&mut self, x: i32, y: i32) -> String {
        if self.is_valid_coordinate(x, y)
            && self.board[x as usize][y as usize] == CellType::Untouched.value()
        {
            self.board[x as usize][y as usize] = Flagged.value();
        }
        format!("{}", self)
    }

    /// DeFlag the coordinate of the board
    ///
    /// # Arguments
    ///
    /// * `x`: x-coordinate
    /// * `y`: y-coordinate
    ///
    /// returns: the board representation
    ///
    /// # Examples
    ///
    /// ```
    ///
    /// ```
    pub fn deflag(&mut self, x: i32, y: i32) -> String {
        if self.is_valid_coordinate(x, y) && self.board[x as usize][y as usize] == Flagged.value() {
            self.board[x as usize][y as usize] = Untouched.value();
        }
        format!("{}", self)
    }

    /// Dig the box
    ///
    /// # Arguments
    ///
    /// * `x`: x-coordinate
    /// * `y`: y-coordinate
    ///
    /// returns: board representation
    ///
    /// # Examples
    ///
    /// ```
    ///
    /// ```
    pub fn dig(&mut self, x: i32, y: i32) -> String {
        if x < 0
            || y < 0
            || x >= self.board.len() as i32
            || y >= self.board[0].len() as i32
            || self.board[x as usize][y as usize] != Untouched.value()
        {
            return format!("{}", self);
        } else if self.bomb[x as usize][y as usize] {
            let x = x as usize;
            let y = y as usize;
            // remove the bomb
            self.bomb[x][y] = false;
            // update number of count
            let surrounding_bombs = self.get_adjacent_bombs(x, y);
            if surrounding_bombs != 0 {
                self.board[x][y] = char::from_digit(surrounding_bombs as u32, 10).unwrap();
            } else {
                self.board[x][y] = Untouched.value();
            }
            self.update_surrounding_count(x, y);
            self.dig(x as i32, y as i32);
            return "BOOM".into();
        }
        self.dig_recursively(x, y);
        self.check_rep();
        format!("{}", self)
    }

    fn dig_recursively(&mut self, x: i32, y: i32) {
        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();

        queue.push_back([x, y]);
        visited.insert(format!("{x},{y}"));

        while let Some(pos) = queue.pop_front() {
            let surrounding_bombs = self.get_adjacent_bombs(pos[0] as usize, pos[1] as usize);
            if self.board[pos[0] as usize][pos[1] as usize] == Flagged.value() {
                continue;
            } else if surrounding_bombs != 0 {
                self.board[pos[0] as usize][pos[1] as usize] =
                    char::from_digit(surrounding_bombs as u32, 10).unwrap();
            } else {
                // No surrounding bombs, uncover surroundings
                if !self.bomb[pos[0] as usize][pos[1] as usize] {
                    self.board[pos[0] as usize][pos[1] as usize] = NoNeighborBombs.value();
                }
                for dir in self.dirs {
                    let _x = dir[0] + pos[0];
                    let _y = dir[1] + pos[1];
                    if self.is_valid_coordinate(_x, _y) && !visited.contains(&format!("{_x},{_y}"))
                    {
                        queue.push_back([_x, _y]);
                        visited.insert(format!("{_x},{_y}"));
                    }
                }
            }
        }
    }

    fn update_surrounding_count(&mut self, x: usize, y: usize) {
        for dir in self.dirs {
            let _x = x as i32 + dir[0];
            let _y = y as i32 + dir[1];

            if self.is_valid_coordinate(_x, _y)
                && self.board[_x as usize][_y as usize] != Untouched.value()
                && self.board[_x as usize][_y as usize] != Flagged.value()
            {
                let _x = _x as usize;
                let _y = _y as usize;
                if self.board[_x][_y] == '1' {
                    self.board[_x][_y] = NoNeighborBombs.value();
                } else {
                    self.board[_x][_y] =
                        char::from_digit(self.board[_x][_y].to_digit(10).unwrap() - 1, 10).unwrap();
                }
            }
        }
        self.check_rep();
    }

    pub fn is_complete(&self) -> bool {
        false
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
