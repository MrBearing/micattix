// src/game.rs - ゲームセッション管理
use crate::core::{Board, BoardSize, Player, Piece};
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
}

impl GameSession {
    pub fn new(size: BoardSize) -> Self {
        let mut scores = HashMap::new();
        scores.insert(Player::First, PlayerScore::new());
        scores.insert(Player::Second, PlayerScore::new());

        let mut total_scores = HashMap::new();
        total_scores.insert(Player::First, 0);
        total_scores.insert(Player::Second, 0);

        Self {
            board: Board::new(size),
            current_player: Player::First,
            scores,
            round: 1,
            total_scores,
        }
    }

    pub fn new_with_board(board: Board) -> Self {
        let mut scores = HashMap::new();
        scores.insert(Player::First, PlayerScore::new());
        scores.insert(Player::Second, PlayerScore::new());

        let mut total_scores = HashMap::new();
        total_scores.insert(Player::First, 0);
        total_scores.insert(Player::Second, 0);

        Self {
            board,
            current_player: Player::First,
            scores,
            round: 1,
            total_scores,
        }
    }

    // プレイヤーの移動を処理
    pub fn process_move(&mut self, target: (usize, usize)) -> Result<(), String> {
        let result = self.board.make_move(self.current_player, target);
        
        match result {
            Ok(piece) => {
                if let Piece::Number(_) = piece {
                    self.scores.get_mut(&self.current_player).unwrap().add_piece(piece);
                }
                self.current_player = self.current_player.next();
                Ok(())
            },
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

        let first_score = self.scores.get(&Player::First).unwrap().total;
        let second_score = self.scores.get(&Player::Second).unwrap().total;

        if first_score > second_score {
            Some(Player::First)
        } else if second_score > first_score {
            Some(Player::Second)
        } else {
            None // 引き分け
        }
    }

    // 次のラウンドを開始
    pub fn start_next_round(&mut self) {
        // 現在のラウンドのスコアを合計に追加
        for player in [Player::First, Player::Second].iter() {
            let round_score = self.scores.get(player).unwrap().total;
            *self.total_scores.get_mut(player).unwrap() += round_score;
        }

        // 新しいラウンドを初期化
        self.board = Board::new(self.board.size);
        self.scores.insert(Player::First, PlayerScore::new());
        self.scores.insert(Player::Second, PlayerScore::new());
        self.round += 1;
        // 先手・後手を入れ替える場合は以下をコメント解除
        // self.current_player = self.current_player.next();
    }

    // 総合勝者を取得
    pub fn get_overall_winner(&self) -> Option<Player> {
        let first_total = self.total_scores.get(&Player::First).unwrap();
        let second_total = self.total_scores.get(&Player::Second).unwrap();

        if first_total > second_total {
            Some(Player::First)
        } else if second_total > first_total {
            Some(Player::Second)
        } else {
            None // 引き分け
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
    pub fn new(size: BoardSize) -> Self {
        Self {
            session: GameSession::new(size),
            listeners: Vec::new(),
        }
    }

    pub fn new_with_board(board: Board) -> Self {
        Self {
            session: GameSession::new_with_board(board),
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
                    let scores = self.session.scores.iter().map(|(k, v)| (*k, v.total)).collect();
                    
                    self.notify(GameEvent::RoundEnded(winner, scores));
                }
            },
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
        self.notify(GameEvent::GameEnded(winner, self.session.total_scores.clone()));
    }
}