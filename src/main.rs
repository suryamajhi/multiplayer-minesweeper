use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::io::Result;
use std::sync::{Arc, Mutex};
use std::thread;
use minesweeper::Board;

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

fn handle_connection(mut stream: TcpStream, board: Arc<Mutex<Board>>) -> Result<()>{
    let mut welcome_message = "Welcome to Minesweeper. Player 1 including you. Type 'help' for help. \n> ";
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
    Ok(())
}

fn handle_request(line: &str) -> String {
    match line {
        "bye" => String::new(),
        "boom" => "BOOM!".to_string(),
        "help" => "Available commands: help, boom, bye".to_string(),
        _ => format!("Received: {}", line)
    }
}
