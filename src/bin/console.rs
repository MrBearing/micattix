use micattix::core::{BoardSize, GameMode};
use micattix::ui::ConsoleUI;
use std::io::{self, Write};

fn main() {
    println!("Welcome to Micattix!");
    
    // ボードサイズを選択
    print!("Select board size (1: 4x4, 2: 6x6): ");
    io::stdout().flush().unwrap();
    
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    
    let size = match input.trim() {
        "1" => BoardSize::Small,
        "2" => BoardSize::Large,
        _ => {
            println!("Invalid selection, using 4x4 board");
            BoardSize::Small
        }
    };
    
    // プレイヤー数を選択
    print!("Select game mode (1: 2 Players, 2: 4 Players): ");
    io::stdout().flush().unwrap();
    
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    
    let game_mode = match input.trim() {
        "1" => GameMode::TwoPlayers,
        "2" => GameMode::FourPlayers,
        _ => {
            println!("Invalid selection, using 2 Players mode");
            GameMode::TwoPlayers
        }
    };
    
    // UIを初期化して実行
    let mut ui = ConsoleUI::new(size, game_mode);
    ui.run();
}