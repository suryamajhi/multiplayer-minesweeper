pub mod board;

use crate::board::Board;
use regex::Regex;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};

pub fn handle_connection(
    mut stream: TcpStream,
    mut board: Arc<Mutex<Board>>,
) -> std::io::Result<()> {
    let player_count = board.lock().unwrap().add_player();
    let col = board.lock().unwrap().get_size_y();
    let row = board.lock().unwrap().get_size_x();

    let mut welcome_message = format!(
        "Welcome to Minesweeper. Player {player_count} including you. Board: {col} columns by {row} rows. Type 'help' for help. \n> ",
    );

    stream.write_all(welcome_message.as_bytes())?;
    stream.flush().unwrap();

    let reader = BufReader::new(stream.try_clone()?);
    for line in reader.lines() {
        let line = line.unwrap();

        let output = handle_request(&line, &mut board);

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

fn handle_request(line: &str, board: &mut Arc<Mutex<Board>>) -> String {
    let help = String::from("Commands: \nlook\ndig [X] [Y]\nflag [X] [Y]\ndeflag [X] [Y]\nbye\n");
    let regex = Regex::new(
        "(look)|(help)|(bye)|(dig -?\\d+ -?\\d+)|(flag -?\\d+ -?\\d+)|(deflag -?\\d+ -?\\d+)",
    )
    .unwrap();
    match regex.is_match(line) {
        false => help, // invalid input
        true => {
            let mut tokens = line.split_whitespace();
            let command_token = tokens.next().unwrap();
            match command_token {
                "help" => help,
                "bye" => "".into(),
                "look" => {
                    format!("{}", board.lock().unwrap())
                }
                &_ => {
                    let x = tokens.next().unwrap().parse::<i32>().unwrap();
                    let y = tokens.next().unwrap().parse::<i32>().unwrap();
                    match command_token {
                        "dig" => board.lock().unwrap().dig(y, x),
                        "flag" => board.lock().unwrap().flag(y, x),
                        "deflag" => board.lock().unwrap().deflag(y, x),
                        &_ => "".into(),
                    }
                }
            }
        }
    }
}
