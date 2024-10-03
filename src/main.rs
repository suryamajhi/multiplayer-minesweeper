use minesweeper::board::Board;
use minesweeper::handle_connection;
use std::io::Result;
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::thread;

fn main() -> Result<()> {
    let board = Arc::new(Mutex::new(Board::new(10, 10)));
    let listener = TcpListener::bind("127.0.0.1:4445").unwrap();
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let board = Arc::clone(&board);

        thread::spawn(move || {
            handle_connection(stream, board).unwrap_or_else(|e| println!("{:?}", e));
        });
    }

    Ok(())
}
