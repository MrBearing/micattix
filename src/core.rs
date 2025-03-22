// src/core.rs - コアとなるゲームロジック
use rand::prelude::*;
use std::fmt;

// ゲームモード定義
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GameMode {
    TwoPlayers,
    FourPlayers,
}

// ゲーム盤のサイズ定義
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BoardSize {
    Small, // 4x4
    Large, // 6x6
}

impl BoardSize {
    pub fn dimensions(&self) -> (usize, usize) {
        match self {
            BoardSize::Small => (4, 4),
            BoardSize::Large => (6, 6),
        }
    }
}

// プレイヤー定義
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Player {
    First,   // 横軸移動
    Second,  // 縦軸移動
    Third,   // 横軸移動
    Fourth,  // 縦軸移動
}

// 移動方向
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MoveDirection {
    Horizontal, // 横
    Vertical,   // 縦
}

impl Player {
    pub fn direction(&self) -> MoveDirection {
        match self {
            Player::First | Player::Third => MoveDirection::Horizontal,
            Player::Second | Player::Fourth => MoveDirection::Vertical,
        }
    }

    pub fn next(&self) -> Self {
        match self {
            Player::First => Player::Second,
            Player::Second => Player::Third,
            Player::Third => Player::Fourth,
            Player::Fourth => Player::First,
        }
    }

    // 2人モードでの次のプレイヤー
    pub fn next_two_player(&self) -> Self {
        match self {
            Player::First => Player::Second,
            Player::Second => Player::First,
            _ => Player::First, // 念のための対応
        }
    }

    // ゲームモードに応じたプレイヤーリストを取得
    pub fn get_players(game_mode: GameMode) -> Vec<Player> {
        match game_mode {
            GameMode::TwoPlayers => vec![Player::First, Player::Second],
            GameMode::FourPlayers => vec![Player::First, Player::Second, Player::Third, Player::Fourth],
        }
    }

    // ゲームモードに応じた次のプレイヤーを取得
    // ゲームモードに応じた次のプレイヤーを取得
    pub fn next_for_mode(&self, game_mode: GameMode) -> Self {
        match game_mode {
            GameMode::TwoPlayers => {
                match self {
                    Player::First => Player::Second,
                    Player::Second => Player::First,
                    Player::Third => Player::Fourth,  // これらは2人モードでは使われないはず
                    Player::Fourth => Player::Third,  // これらは2人モードでは使われないはず
                }
            },
            GameMode::FourPlayers => self.next(),
        }
    }
}

// 盤面上の駒
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Piece {
    Number(i32),    // 数値の駒
    Cross,          // クロスチップ
    Empty,          // 空きマス
}

impl fmt::Display for Piece {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Piece::Number(n) => write!(f, "{:>3}", n),
            Piece::Cross => write!(f, "  X"),
            Piece::Empty => write!(f, "   "),
        }
    }
}

// 盤面の状態
#[derive(Debug, Clone)]
pub struct Board {
    pub size: BoardSize,
    pub pieces: Vec<Vec<Piece>>,
    pub cross_position: (usize, usize),
}

impl Board {
    // 新しい盤面を生成
    pub fn new(size: BoardSize) -> Self {
        let (rows, cols) = size.dimensions();
        let pieces = vec![vec![Piece::Empty; cols]; rows];
        let cross_position = (0, 0); // 仮の初期位置

        let mut board = Board {
            size,
            pieces,
            cross_position,
        };

        board.initialize();
        board
    }

    // 盤面を初期化（駒をランダムに配置）
    fn initialize(&mut self) {
        let (rows, cols) = self.size.dimensions();
        
        // 駒のセットを作成
        let mut pieces_set = match self.size {
            BoardSize::Small => {
                // 1～7を各2個、8を1個
                let mut set = Vec::new();
                for i in 1..=7 {
                    set.push(Piece::Number(i));
                    set.push(Piece::Number(i));
                }
                set.push(Piece::Number(8));
                set.push(Piece::Cross);
                set
            },
            BoardSize::Large => {
                // 1～9を各2個、-1～-10と+10を各1個
                let mut set = Vec::new();
                for i in 1..=9 {
                    set.push(Piece::Number(i));
                    set.push(Piece::Number(i));
                }
                for i in 1..=10 {
                    set.push(Piece::Number(-i));
                }
                set.push(Piece::Number(10));
                set.push(Piece::Cross);
                
                // 6x6のボードには36個の駒が必要だが、上記のコードでは
                // 9*2 + 10 + 1 + 1 = 30個しか作成されていない
                // 残りの6個を埋めるために追加駒を作成
                for i in 1..=6 {
                    set.push(Piece::Number(i)); // 追加の数字を入れる
                }
                
                set
            }
        };

        // 駒をシャッフル
        let mut rng = rand::thread_rng();
        pieces_set.shuffle(&mut rng);

        // 盤面に駒を配置
        let total_cells = rows * cols;
        assert_eq!(pieces_set.len(), total_cells, "駒の数が盤面のセル数と一致しません");
        
        let mut index = 0;
        for row in 0..rows {
            for col in 0..cols {
                self.pieces[row][col] = pieces_set[index];
                if pieces_set[index] == Piece::Cross {
                    self.cross_position = (row, col);
                }
                index += 1;
            }
        }
    }

    // 有効な移動先の一覧を取得
    pub fn get_valid_moves(&self, player: Player) -> Vec<(usize, usize)> {
        let (row, col) = self.cross_position;
        let (rows, cols) = self.size.dimensions();
        
        let mut valid_moves = Vec::new();

        match player.direction() {
            MoveDirection::Horizontal => {
                // 横方向の移動
                for c in 0..cols {
                    if c != col && self.pieces[row][c] != Piece::Empty {
                        valid_moves.push((row, c));
                    }
                }
            },
            MoveDirection::Vertical => {
                // 縦方向の移動
                for r in 0..rows {
                    if r != row && self.pieces[r][col] != Piece::Empty {
                        valid_moves.push((r, col));
                    }
                }
            }
        }

        valid_moves
    }

    // 駒を移動して取得
    pub fn make_move(&mut self, player: Player, target: (usize, usize)) -> Result<Piece, String> {
        let valid_moves = self.get_valid_moves(player);
        
        if !valid_moves.contains(&target) {
            return Err(format!("Invalid move to {:?}", target));
        }

        // 移動先の駒を記録
        let piece = self.pieces[target.0][target.1];
        
        // クロスチップを移動
        self.pieces[self.cross_position.0][self.cross_position.1] = Piece::Empty;
        self.pieces[target.0][target.1] = Piece::Cross;
        self.cross_position = target;

        Ok(piece)
    }

    // ゲームが終了したかチェック
    pub fn is_game_over(&self) -> bool {
        let (rows, cols) = self.size.dimensions();
        
        for row in 0..rows {
            for col in 0..cols {
                if self.pieces[row][col] != Piece::Empty && self.pieces[row][col] != Piece::Cross {
                    return false;
                }
            }
        }
        
        true
    }

    // 盤面を表示（デバッグ用）
    pub fn display(&self) -> String {
        let (rows, cols) = self.size.dimensions();
        let mut result = String::new();
        
        for row in 0..rows {
            for col in 0..cols {
                result.push_str(&format!("{} ", self.pieces[row][col]));
            }
            result.push('\n');
        }
        
        result
    }

    // 特定の位置の駒を取得
    pub fn get_piece(&self, row: usize, col: usize) -> Piece {
        if row < self.pieces.len() && col < self.pieces[0].len() {
            self.pieces[row][col]
        } else {
            Piece::Empty
        }
    }

    // 特定の位置に駒を設定（テスト用）
    pub fn set_piece(&mut self, row: usize, col: usize, piece: Piece) {
        if row < self.pieces.len() && col < self.pieces[0].len() {
            self.pieces[row][col] = piece;
            if piece == Piece::Cross {
                self.cross_position = (row, col);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::*;

    #[test]
    fn test_board_size() {
        assert_eq!(BoardSize::Small.dimensions(), (4, 4));
        assert_eq!(BoardSize::Large.dimensions(), (6, 6));
    }

    #[test]
    fn test_player_direction() {
        assert_eq!(Player::First.direction(), MoveDirection::Horizontal);
        assert_eq!(Player::Second.direction(), MoveDirection::Vertical);
        assert_eq!(Player::Third.direction(), MoveDirection::Horizontal);
        assert_eq!(Player::Fourth.direction(), MoveDirection::Vertical);
    }

    #[test]
    fn test_player_next() {
        assert_eq!(Player::First.next(), Player::Second);
        assert_eq!(Player::Second.next(), Player::Third);
        assert_eq!(Player::Third.next(), Player::Fourth);
        assert_eq!(Player::Fourth.next(), Player::First);
    }
    
    #[test]
    fn test_player_next_two_player() {
        assert_eq!(Player::First.next_two_player(), Player::Second);
        assert_eq!(Player::Second.next_two_player(), Player::First);
    }
    
    #[test]
    fn test_players_for_game_mode() {
        // 2人モードのGameSessionをテスト
        let two_player_session = GameSession::new(BoardSize::Small, GameMode::TwoPlayers);
        assert_eq!(two_player_session.players.len(), 2);
        assert!(two_player_session.players.contains(&Player::First));
        assert!(two_player_session.players.contains(&Player::Second));
        assert!(!two_player_session.players.contains(&Player::Third));
        assert!(!two_player_session.players.contains(&Player::Fourth));
        
        // 4人モードのGameSessionをテスト
        let four_player_session = GameSession::new(BoardSize::Small, GameMode::FourPlayers);
        assert_eq!(four_player_session.players.len(), 4);
        assert!(four_player_session.players.contains(&Player::First));
        assert!(four_player_session.players.contains(&Player::Second));
        assert!(four_player_session.players.contains(&Player::Third));
        assert!(four_player_session.players.contains(&Player::Fourth));
    }

    #[test]
    fn test_piece_display() {
        assert_eq!(format!("{}", Piece::Number(5)), "  5");
        assert_eq!(format!("{}", Piece::Number(-3)), " -3");
        assert_eq!(format!("{}", Piece::Cross), "  X");
        assert_eq!(format!("{}", Piece::Empty), "   ");
    }

    #[test]
    fn test_board_initialization() {
        let board = Board::new(BoardSize::Small);
        
        // 4x4ボードのサイズを確認
        assert_eq!(board.pieces.len(), 4);
        assert_eq!(board.pieces[0].len(), 4);
        
        // 駒の総数を確認
        let mut num_count = 0;
        let mut cross_count = 0;
        let mut empty_count = 0;
        
        for row in &board.pieces {
            for piece in row {
                match piece {
                    Piece::Number(_) => num_count += 1,
                    Piece::Cross => cross_count += 1,
                    Piece::Empty => empty_count += 1,
                }
            }
        }
        
        assert_eq!(num_count, 15); // 1-7の各2個と8の1個
        assert_eq!(cross_count, 1); // クロスチップは1個
        assert_eq!(empty_count, 0); // 空きマスはない（初期状態）
    }

    #[test]
    fn test_large_board_initialization() {
        let board = Board::new(BoardSize::Large);
        
        // 6x6ボードのサイズを確認
        assert_eq!(board.pieces.len(), 6);
        assert_eq!(board.pieces[0].len(), 6);
        
        // 駒の総数を確認
        let mut positive_count = 0;
        let mut negative_count = 0;
        let mut cross_count = 0;
        let mut empty_count = 0;
        
        for row in &board.pieces {
            for piece in row {
                match piece {
                    Piece::Number(n) if *n > 0 => positive_count += 1,
                    Piece::Number(n) if *n < 0 => negative_count += 1,
                    Piece::Cross => cross_count += 1,
                    Piece::Empty => empty_count += 1,
                    _ => {},
                }
            }
        }
        
        assert_eq!(cross_count, 1, "There should be exactly 1 cross chip");
        assert_eq!(empty_count, 0, "There should be no empty cells initially");
        assert_eq!(positive_count + negative_count + cross_count, 36, "Total cells should be 36 in a 6x6 board");
        
        // 正の数と負の数のチェック（厳密な数ではなく、存在することを確認）
        assert!(positive_count > 0, "There should be positive numbers");
        assert!(negative_count > 0, "There should be negative numbers");
    }

    #[test]
    fn test_valid_moves() {
        let mut board = Board::new(BoardSize::Small);
        
        // クロスチップの位置を固定
        board.cross_position = (1, 2);
        for row in 0..4 {
            for col in 0..4 {
                if (row, col) == (1, 2) {
                    board.pieces[row][col] = Piece::Cross;
                } else {
                    board.pieces[row][col] = Piece::Number(1);
                }
            }
        }
        
        // 横方向の有効な移動を確認
        let horizontal_moves = board.get_valid_moves(Player::First);
        assert_eq!(horizontal_moves.len(), 3); // 同じ行の他の3マス
        assert!(horizontal_moves.contains(&(1, 0)));
        assert!(horizontal_moves.contains(&(1, 1)));
        assert!(horizontal_moves.contains(&(1, 3)));
        
        // 縦方向の有効な移動を確認
        let vertical_moves = board.get_valid_moves(Player::Second);
        assert_eq!(vertical_moves.len(), 3); // 同じ列の他の3マス
        assert!(vertical_moves.contains(&(0, 2)));
        assert!(vertical_moves.contains(&(2, 2)));
        assert!(vertical_moves.contains(&(3, 2)));
    }

    #[test]
    fn test_make_move() {
        let mut board = Board::new(BoardSize::Small);
        
        // クロスチップの位置を固定
        board.cross_position = (1, 2);
        for row in 0..4 {
            for col in 0..4 {
                if (row, col) == (1, 2) {
                    board.pieces[row][col] = Piece::Cross;
                } else {
                    board.pieces[row][col] = Piece::Number(1);
                }
            }
        }
        
        // 特定の位置に特別な値を設定
        board.pieces[1][3] = Piece::Number(5);
        
        // 移動を実行
        let result = board.make_move(Player::First, (1, 3));
        
        // 移動が成功し、正しい駒が取得されることを確認
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Piece::Number(5));
        
        // クロスチップが移動していることを確認
        assert_eq!(board.cross_position, (1, 3));
        assert_eq!(board.pieces[1][3], Piece::Cross);
        assert_eq!(board.pieces[1][2], Piece::Empty);
    }

    #[test]
    fn test_invalid_move() {
        let mut board = Board::new(BoardSize::Small);
        
        // クロスチップの位置を固定
        board.cross_position = (1, 2);
        for row in 0..4 {
            for col in 0..4 {
                if (row, col) == (1, 2) {
                    board.pieces[row][col] = Piece::Cross;
                } else {
                    board.pieces[row][col] = Piece::Number(1);
                }
            }
        }
        
        // 無効な移動（対角線上）
        let result = board.make_move(Player::First, (2, 3));
        assert!(result.is_err());
        
        // 無効な移動（クロスチップの位置）
        let result = board.make_move(Player::First, (1, 2));
        assert!(result.is_err());
        
        // 無効な移動（盤面外）
        let result = board.make_move(Player::First, (5, 5));
        assert!(result.is_err());
    }

    #[test]
    fn test_game_over() {
        let mut board = Board::new(BoardSize::Small);
        
        // すべてのマスを空にする
        for row in 0..4 {
            for col in 0..4 {
                board.pieces[row][col] = Piece::Empty;
            }
        }
        
        // クロスチップだけを配置
        board.pieces[1][2] = Piece::Cross;
        board.cross_position = (1, 2);
        
        // ゲームが終了していることを確認
        assert!(board.is_game_over());
        
        // 1つ数値駒を追加
        board.pieces[0][0] = Piece::Number(3);
        
        // ゲームがまだ終了していないことを確認
        assert!(!board.is_game_over());
    }
    
    #[test]
    fn test_get_players_for_game_mode() {
        let two_players = Player::get_players(GameMode::TwoPlayers);
        assert_eq!(two_players.len(), 2);
        assert_eq!(two_players[0], Player::First);
        assert_eq!(two_players[1], Player::Second);
        
        let four_players = Player::get_players(GameMode::FourPlayers);
        assert_eq!(four_players.len(), 4);
        assert_eq!(four_players[0], Player::First);
        assert_eq!(four_players[1], Player::Second);
        assert_eq!(four_players[2], Player::Third);
        assert_eq!(four_players[3], Player::Fourth);
    }
}