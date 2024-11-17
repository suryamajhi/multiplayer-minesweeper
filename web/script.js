function initBoard(event) {
    let response = JSON.parse(event.data);

    let board = response.board;
    let rows = response.size_x;
    let cols = response.size_y;
    let bomb = response.bomb;
    const boardContainer = document.getElementById("welcome-message");

    if (response.message === "BOOM") {
        boardContainer.innerHTML = `<h1 class="text-center text-2xl">Game over. You dig the bomb. Refresh to start again</h1>`
    } else if (response.message === "WIN") {
        boardContainer.innerHTML = `<h1 class="text-center text-2xl">Game is complete.Congratulations on the team work..</h1>`
    } else {
        boardContainer.innerHTML = `<h1 class="text-center text-2xl">Welcome to Minesweeper. Player ${response.player_count} including you. Board: ${cols} columns by ${rows} rows.</h1>`
    }
    const gameBoard = document.getElementById('game-board');
    gameBoard.innerHTML = '';

    // Create the grid
    for (let i = 0; i < rows; i++) {
        for (let j = 0; j < cols; j++) {
            const cell = document.createElement('div');
            cell.classList.add('cell');
            cell.dataset.row = i;
            cell.dataset.col = j;
            cell.addEventListener('click', () => {
                event.srcElement.send(`dig ${j} ${i}`)
            });
            cell.addEventListener('contextmenu', (e) => {
                e.preventDefault();
                e.stopPropagation();
                if(cell.classList.contains("flag")) {
                    event.srcElement.send(`deflag ${j} ${i}`)
                } else {
                    event.srcElement.send(`flag ${j} ${i}`)
                }
            });
            gameBoard.appendChild(cell);
            if (board[i][j] === 'F') {
                cell.classList.add('flag');
                cell.textContent = '🇳🇵';
            } else if (board[i][j] !== '-') {
                cell.classList.add('revealed');
                cell.textContent = board[i][j];
            }

            if(response.message === "BOOM") {
                if(bomb[i][j]) {
                    cell.classList.add('mine');
                    cell.textContent = '💣';
                }
            }
        }
    }
}
