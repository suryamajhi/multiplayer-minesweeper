pub mod board;

use crate::board::Board;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};

pub fn handle_connection(mut stream: TcpStream, board: Arc<Mutex<Board>>) -> std::io::Result<()> {
    let player_count = board.lock().unwrap().add_player();
    let mut welcome_message = format!(
        "Welcome to Minesweeper. Player {player_count} including you. Type 'help' for help. \n> ");
    stream.write_all(welcome_message.as_bytes())?;
    stream.flush().unwrap();

    let reader = BufReader::new(stream.try_clone()?);
    for line in reader.lines() {
        let line = line.unwrap();

        let output = handle_request(&line);

        match output.as_str() {
            "" => break,
            s if s.starts_with("BOOM") => {
                stream.write_all(output.as_bytes()).unwrap();
                break;
            }
            _ => {
                stream.write_all(output.as_bytes()).unwrap();
                stream.write_all("\n".as_bytes()).unwrap();
                stream.flush().unwrap();
                stream.write_all("> ".as_bytes()).unwrap()
            }
        }
    }

    board.lock().unwrap().remove_player();
    Ok(())
}

fn handle_request(line: &str) -> String {
    match line {
        "bye" => String::new(),
        "boom" => "BOOM!".to_string(),
        "help" => "Available commands: help, boom, bye".to_string(),
        _ => format!("Received: {}", line),
    }
}
