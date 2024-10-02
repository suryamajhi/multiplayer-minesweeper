use std::sync::Mutex;

pub struct Board {
    players: usize,
    size_x: usize,
    size_y: usize
}

impl Board {
    pub fn new(size_x: usize, size_y: usize) -> Self {
        Board {
            players: 0,
            size_x,
            size_y
        }
    }

    fn add_player(&mut self) -> usize {
        self.players = self.players + 1;
        self.players
    }

    fn remove_player(&mut self) {
        self.players = self.players - 1;
    }
}