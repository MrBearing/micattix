// src/ui.rs - UI関連のコード
use crate::core::{Board, BoardSize, GameMode, Player};
use crate::game::{GameEvent, GameEventListener, GameManager};
use std::io::{self, Write};

// コンソールUI
pub struct ConsoleUI {
    manager: GameManager,
}

impl ConsoleUI {
    pub fn new(size: BoardSize, game_mode: GameMode) -> Self {
        let manager = GameManager::new(size, game_mode);

        // セルフを登録できないのでここではリスナーは登録しない
        // ゲーム開始後に別途登録する

        Self { manager }
    }

    pub fn run(&mut self) {
        // ゲーム開始
        self.manager.start_game();

        loop {
            // 盤面表示
            println!("{}", self.manager.session.board.display());

            // 現在のプレイヤーとスコアを表示
            let current = self.manager.session.current_player;
            println!("Current player: {:?}", current);

            for player in [Player::First, Player::Second].iter() {
                let score = &self.manager.session.scores[player];
                println!("{:?} score: {}", player, score.total);
            }

            // 有効な移動を表示
            let valid_moves = self.manager.session.board.get_valid_moves(current);
            println!("Valid moves: {:?}", valid_moves);

            // 入力受付
            print!("Enter move (row,col): ");
            io::stdout().flush().unwrap();

            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap();

            let input = input.trim();
            if input == "quit" {
                break;
            }

            // 入力をパース
            let coords: Vec<&str> = input.split(',').collect();
            if coords.len() != 2 {
                println!("Invalid input! Enter as 'row,col'");
                continue;
            }

            let row = coords[0].trim().parse::<usize>();
            let col = coords[1].trim().parse::<usize>();

            if row.is_err() || col.is_err() {
                println!("Invalid coordinates!");
                continue;
            }

            let target = (row.unwrap(), col.unwrap());

            // 移動実行
            self.manager.make_move(target);

            // ラウンド終了チェック
            if self.manager.session.is_round_over() {
                println!("Round {} ended!", self.manager.session.round);

                match self.manager.session.get_round_winner() {
                    Some(winner) => println!("Winner: {:?}", winner),
                    None => println!("It's a draw!"),
                }

                print!("Start next round? (y/n): ");
                io::stdout().flush().unwrap();

                let mut input = String::new();
                io::stdin().read_line(&mut input).unwrap();

                if input.trim().to_lowercase() == "y" {
                    self.manager.start_next_round();
                } else {
                    self.manager.end_game();
                    break;
                }
            }
        }

        // ゲーム終了時の総合結果
        println!("Game over!");
        println!("Final scores:");
        for player in [Player::First, Player::Second].iter() {
            println!(
                "{:?}: {}",
                player, self.manager.session.total_scores[player]
            );
        }

        match self.manager.session.get_overall_winner() {
            Some(winner) => println!("Overall winner: {:?}", winner),
            None => println!("Overall result: Draw"),
        }
    }
}

impl GameEventListener for ConsoleUI {
    fn on_event(&mut self, event: GameEvent) {
        match event {
            GameEvent::GameStarted => {
                println!("Game started!");
            }
            GameEvent::RoundStarted(round) => {
                println!("Round {} started!", round);
            }
            GameEvent::MoveMade(player, target, piece) => {
                println!("{:?} moved to {:?} and got {:?}", player, target, piece);
            }
            GameEvent::InvalidMove(player, target, reason) => {
                println!("Invalid move by {:?} to {:?}: {}", player, target, reason);
            }
            GameEvent::RoundEnded(winner, scores) => {
                println!("Round ended!");
                for (player, score) in scores {
                    println!("{:?} score: {}", player, score);
                }
                match winner {
                    Some(w) => println!("Winner: {:?}", w),
                    None => println!("Round ended in a draw"),
                }
            }
            GameEvent::GameEnded(winner, scores) => {
                println!("Game ended!");
                for (player, score) in scores {
                    println!("{:?} total score: {}", player, score);
                }
                match winner {
                    Some(w) => println!("Overall winner: {:?}", w),
                    None => println!("Game ended in a draw"),
                }
            }
        }
    }
}

// GUIのトレイトを定義（将来的な拡張用）
pub trait GUI {
    fn init(&mut self);
    fn update(&mut self, board: &Board);
    fn get_move(&mut self) -> (usize, usize);
    fn show_message(&mut self, message: &str);
    fn close(&mut self);
}
