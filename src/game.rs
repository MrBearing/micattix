// src/game.rs - ゲームセッション管理
use crate::core::{Board, BoardSize, GameMode, Piece, Player};
use std::collections::HashMap;

// プレイヤースコア
#[derive(Debug, Clone, Default)]
pub struct PlayerScore {
    pub pieces: Vec<Piece>,
    pub total: i32,
}

impl PlayerScore {
    pub fn new() -> Self {
        Self {
            pieces: Vec::new(),
            total: 0,
        }
    }

    pub fn add_piece(&mut self, piece: Piece) {
        if let Piece::Number(value) = piece {
            self.total += value;
        }
        self.pieces.push(piece);
    }
}

// ゲームセッション
#[derive(Debug, Clone)]
pub struct GameSession {
    pub board: Board,
    pub current_player: Player,
    pub scores: HashMap<Player, PlayerScore>,
    pub round: usize,
    pub total_scores: HashMap<Player, i32>,
    pub game_mode: GameMode,
    pub players: Vec<Player>,
}

impl GameSession {
    pub fn new(size: BoardSize, game_mode: GameMode) -> Self {
        let mut scores = HashMap::new();
        let mut total_scores = HashMap::new();

        // ゲームモードに応じたプレイヤーリスト
        let players = match game_mode {
            GameMode::TwoPlayers => vec![Player::First, Player::Second],
            GameMode::FourPlayers => {
                vec![Player::First, Player::Second, Player::Third, Player::Fourth]
            }
        };

        for player in &players {
            scores.insert(*player, PlayerScore::new());
            total_scores.insert(*player, 0);
        }

        Self {
            board: Board::new(size),
            current_player: Player::First,
            scores,
            round: 1,
            total_scores,
            game_mode,
            players,
        }
    }

    pub fn new_with_board(board: Board, game_mode: GameMode) -> Self {
        let mut scores = HashMap::new();
        let mut total_scores = HashMap::new();

        // ゲームモードに応じたプレイヤーリスト
        let players = match game_mode {
            GameMode::TwoPlayers => vec![Player::First, Player::Second],
            GameMode::FourPlayers => {
                vec![Player::First, Player::Second, Player::Third, Player::Fourth]
            }
        };

        for player in &players {
            scores.insert(*player, PlayerScore::new());
            total_scores.insert(*player, 0);
        }

        Self {
            board,
            current_player: Player::First,
            scores,
            round: 1,
            total_scores,
            game_mode,
            players,
        }
    }

    // プレイヤーの移動を処理
    pub fn process_move(&mut self, target: (usize, usize)) -> Result<(), String> {
        let result = self.board.make_move(self.current_player, target);

        match result {
            Ok(piece) => {
                if let Piece::Number(_) = piece {
                    self.scores
                        .get_mut(&self.current_player)
                        .unwrap()
                        .add_piece(piece);
                }
                self.current_player = self.current_player.next_for_mode(self.game_mode);
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    // ゲームが終了したか確認
    pub fn is_round_over(&self) -> bool {
        self.board.is_game_over()
    }

    // 現在のラウンドの勝者を取得
    pub fn get_round_winner(&self) -> Option<Player> {
        if !self.is_round_over() {
            return None;
        }

        // 全プレイヤーの中で最高得点を見つける
        let mut highest_score = i32::MIN;
        let mut winner: Option<Player> = None;
        let mut is_tie = false;

        for player in &self.players {
            let score = self.scores.get(player).unwrap().total;

            if score > highest_score {
                highest_score = score;
                winner = Some(*player);
                is_tie = false;
            } else if score == highest_score {
                is_tie = true;
            }
        }

        if is_tie {
            None // 引き分け
        } else {
            winner
        }
    }

    // 次のラウンドを開始
    pub fn start_next_round(&mut self) {
        // 現在のラウンドのスコアを合計に追加
        for player in &self.players {
            let round_score = self.scores.get(player).unwrap().total;
            *self.total_scores.get_mut(player).unwrap() += round_score;
        }

        // 新しいラウンドを初期化
        self.board = Board::new(self.board.size);

        // スコアを初期化
        for player in &self.players {
            self.scores.insert(*player, PlayerScore::new());
        }

        // ラウンドをインクリメント
        self.round += 1;

        // 先手を変える場合はここで設定
        // 例: 4人モードで順番にスタートプレイヤーをローテーション
        if self.round > 1 {
            let index = (self.round - 1) % self.players.len();
            self.current_player = self.players[index];
        }
    }

    // 総合勝者を取得
    pub fn get_overall_winner(&self) -> Option<Player> {
        // 全プレイヤーの中で最高の合計得点を見つける
        let mut highest_total = i32::MIN;
        let mut winner: Option<Player> = None;
        let mut is_tie = false;

        for player in &self.players {
            let total = *self.total_scores.get(player).unwrap();

            if total > highest_total {
                highest_total = total;
                winner = Some(*player);
                is_tie = false;
            } else if total == highest_total {
                is_tie = true;
            }
        }

        if is_tie {
            None // 引き分け
        } else {
            winner
        }
    }

    // 特定のプレイヤーの名前を取得
    pub fn get_player_name(&self, player: Player) -> String {
        match self.game_mode {
            GameMode::TwoPlayers => {
                match player {
                    Player::First => "プレイヤー1 (横)".to_string(),
                    Player::Second => "プレイヤー2 (縦)".to_string(),
                    _ => format!("{:?}", player), // 2人プレイの場合は通常使用されない
                }
            }
            GameMode::FourPlayers => match player {
                Player::First => "プレイヤー1 (横)".to_string(),
                Player::Second => "プレイヤー2 (縦)".to_string(),
                Player::Third => "プレイヤー3 (横)".to_string(),
                Player::Fourth => "プレイヤー4 (縦)".to_string(),
            },
        }
    }
}

// ゲームイベントを表すenum
#[derive(Debug, Clone)]
pub enum GameEvent {
    GameStarted,
    RoundStarted(usize),
    MoveMade(Player, (usize, usize), Piece),
    InvalidMove(Player, (usize, usize), String),
    RoundEnded(Option<Player>, HashMap<Player, i32>),
    GameEnded(Option<Player>, HashMap<Player, i32>),
}

// ゲームイベントのリスナー
pub trait GameEventListener {
    fn on_event(&mut self, event: GameEvent);
}

// ゲームイベントを通知するゲームマネージャー
pub struct GameManager {
    pub session: GameSession,
    listeners: Vec<Box<dyn GameEventListener>>,
}

impl GameManager {
    pub fn new(size: BoardSize, game_mode: GameMode) -> Self {
        Self {
            session: GameSession::new(size, game_mode),
            listeners: Vec::new(),
        }
    }

    pub fn new_with_board(board: Board, game_mode: GameMode) -> Self {
        Self {
            session: GameSession::new_with_board(board, game_mode),
            listeners: Vec::new(),
        }
    }

    pub fn add_listener(&mut self, listener: Box<dyn GameEventListener>) {
        self.listeners.push(listener);
    }

    fn notify(&mut self, event: GameEvent) {
        for listener in &mut self.listeners {
            listener.on_event(event.clone());
        }
    }

    pub fn start_game(&mut self) {
        self.notify(GameEvent::GameStarted);
        self.notify(GameEvent::RoundStarted(self.session.round));
    }

    pub fn make_move(&mut self, target: (usize, usize)) {
        let current_player = self.session.current_player;

        match self.session.process_move(target) {
            Ok(()) => {
                // 移動した駒を取得
                let pieces = &self.session.scores.get(&current_player).unwrap().pieces;
                let last_piece = pieces.last().unwrap_or(&Piece::Empty);

                self.notify(GameEvent::MoveMade(current_player, target, *last_piece));

                // ラウンド終了チェック
                if self.session.is_round_over() {
                    let winner = self.session.get_round_winner();
                    let scores = self
                        .session
                        .scores
                        .iter()
                        .map(|(k, v)| (*k, v.total))
                        .collect();

                    self.notify(GameEvent::RoundEnded(winner, scores));
                }
            }
            Err(e) => {
                self.notify(GameEvent::InvalidMove(current_player, target, e));
            }
        }
    }

    pub fn start_next_round(&mut self) {
        self.session.start_next_round();
        self.notify(GameEvent::RoundStarted(self.session.round));
    }

    pub fn end_game(&mut self) {
        let winner = self.session.get_overall_winner();
        self.notify(GameEvent::GameEnded(
            winner,
            self.session.total_scores.clone(),
        ));
    }
}
