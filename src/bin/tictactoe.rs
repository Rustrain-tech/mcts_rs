use mcts_rs::mcts::{Action, GameState, MCTS};

// ここからは3目並べの実装
#[derive(Clone, Copy, PartialEq, Eq)]
enum Player {
    X, // 先手
    O, // 後手
}

struct TicTacToe {
    board: [Option<Player>; 9], // 盤面
    player: Player,             // 手番のプレイヤー
}

impl TicTacToe {
    fn new() -> TicTacToe {
        TicTacToe {
            board: [None; 9],
            player: Player::X,
        }
    }
}

impl GameState for TicTacToe {
    fn get_legal_moves(&self) -> Vec<Action> {
        self.board
            .iter()
            .enumerate()
            .filter(|(_, &cell)| cell.is_none())
            .map(|(index, _)| index)
            .collect()
    }

    fn make_move(&mut self, action: Action) {
        self.board[action] = Some(self.player);
        self.player = match self.player {
            Player::X => Player::O,
            Player::O => Player::X,
        };
    }

    fn is_terminal(&self) -> bool {
        self.get_winner().is_some() || self.board.iter().all(|&cell| cell.is_some())
    }

    fn get_winner(&self) -> Option<i32> {
        let lines = [
            // 横
            [0, 1, 2],
            [3, 4, 5],
            [6, 7, 8],
            // 縦
            [0, 3, 6],
            [1, 4, 7],
            [2, 5, 8],
            // 斜め
            [0, 4, 8],
            [2, 4, 6],
        ];
        for line in lines.iter() {
            if let Some(player) = self.board[line[0]] {
                if line.iter().all(|&index| self.board[index] == Some(player)) {
                    return Some(match player {
                        Player::X => 1,
                        Player::O => -1,
                    });
                }
            }
        }
        if self.board.iter().all(|&cell| cell.is_some()) {
            return Some(0);
        }
        None
    }

    fn clone(&self) -> Box<dyn GameState> {
        Box::new(TicTacToe {
            board: self.board,
            player: self.player,
        })
    }
}

fn main() {
    let initial_state = Box::new(TicTacToe::new());
    let mut mcts = MCTS::new(initial_state);

    let mut game = TicTacToe::new();

    while !game.is_terminal() {
        let best_move = mcts.get_best_move(10000);
        println!("Best move: {}", best_move);
        game.make_move(best_move);
        print_board(&game);

        if !game.is_terminal() {
            println!("Enter your action (<row>-<column>):");
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).unwrap();
            let row = input.trim().chars().next().unwrap().to_digit(10).unwrap() as usize - 1;
            let column = input.trim().chars().nth(2).unwrap().to_digit(10).unwrap() as usize - 1;
            let action = row * 3 + column;
            game.make_move(action);
            print_board(&game);
        }

        mcts = MCTS::new(game.clone());
    }

    match game.get_winner() {
        Some(1) => println!("X wins!"),
        Some(-1) => println!("O wins!"),
        Some(0) => println!("It's a draw!"),
        _ => unreachable!(),
    }
}

fn print_board(game: &TicTacToe) {
    print!("  ");
    for i in 1..4 {
        print!("{} ", i);
    }
    println!();

    for i in 0..3 {
        print!("{} ", i + 1);
        for j in 0..3 {
            match game.board[i * 3 + j] {
                Some(Player::X) => print!("X "),
                Some(Player::O) => print!("O "),
                None => print!("- "),
            }
        }
        println!();
    }
    println!();
}
