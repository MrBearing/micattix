use micattix::core::BoardSize;
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
    
    // UIを初期化して実行
    let mut ui = ConsoleUI::new(size);
    ui.run();
}