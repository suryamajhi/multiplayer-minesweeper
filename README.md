# Multiplayer Minesweeper:
Project Formulation: https://ocw.mit.edu/ans7870/6/6.005/s16/psets/ps4/

# Usage
1. Run the server
    ```bash
    cargo run 
    ```
2. Connect as client (also from multiple terminals)
    ```bash
    nc 127.0.0.1 4445
    ```
3. Enter Cmd to play minesweeper <br>
    Available commands
   1. look
   2. dig x y
   3. flag x y
   4. deflag x y
   5. help

## TODO! Improvements 
1. Make a webpage UI for client.
2. Implement realtime view of the board.