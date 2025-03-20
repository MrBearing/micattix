#[cfg(test)]
mod integration_tests {
    use micattix::core::{Board, BoardSize, Player, Piece};
    use micattix::game::{GameManager, GameEvent, GameEventListener};
    use std::collections::VecDeque;

    // シンプルなイベントリスナー
    struct SimpleEventRecorder {
        events: VecDeque<GameEvent>,
    }

    impl SimpleEventRecorder {
        fn new() -> Self {
            Self { events: VecDeque::new() }
        }
    }

    impl GameEventListener for SimpleEventRecorder {
        fn on_event(&mut self, event: GameEvent) {
            self.events.push_back(event);
        }
    }

    #[test]
    fn test_game_flow() {
        // GameManagerの初期化
        let mut manager = GameManager::new(BoardSize::Small);
        
        // イベントレコーダーを追加
        let mut recorder = SimpleEventRecorder::new();
        
        // ゲーム開始（リスナーなしで）
        manager.start_game();
        
        // 有効な移動先を取得
        let valid_moves = manager.session.board.get_valid_moves(Player::First);
        assert!(!valid_moves.is_empty());
        
        // 移動を実行
        let move_target = valid_moves[0];
        manager.make_move(move_target);
        
        // 次のプレイヤーをチェック
        assert_eq!(manager.session.current_player, Player::Second);
    }

    #[test]
    fn test_game_round_completion() {
        // 小さい盤面で作業するためのカスタム盤面を設定
        let mut board = Board::new(BoardSize::Small);
        
        // クロスチップの位置を取得
        let cross_pos = board.cross_position;
        
        // まず盤面をすべて空にする
        for row in 0..4 {
            for col in 0..4 {
                if (row, col) != cross_pos {
                    board.set_piece(row, col, Piece::Empty);
                }
            }
        }
        
        // クロスチップが(row, col)にあるとして、その位置に基づいて駒を配置
        
        // クロスチップが最初のプレイヤー（横移動）のための駒を配置
        // 必ず同じ行の別の位置に配置
        let target_col = (cross_pos.1 + 1) % 4; // 違う列を選択
        board.set_piece(cross_pos.0, target_col, Piece::Number(5)); // 同じ行の違う列
        
        // ゲームマネージャーを初期化
        let mut manager = GameManager::new_with_board(board);
        
        // 先手（横移動）に設定
        manager.session.current_player = Player::First;
        
        // ゲーム開始
        manager.start_game();
        
        // 有効な移動先を確認
        let valid_moves = manager.session.board.get_valid_moves(manager.session.current_player);
        
        // デバッグ情報
        println!("Cross position: {:?}", cross_pos);
        println!("Valid moves: {:?}", valid_moves);
        println!("Current player: {:?}", manager.session.current_player);
        println!("Board state:");
        println!("{}", manager.session.board.display());
        
        assert_eq!(valid_moves.len(), 1, "Should have exactly one valid move");
        
        // 移動を実行
        let target = valid_moves[0];
        manager.make_move(target);
        
        // ラウンド終了をチェック
        assert!(manager.session.is_round_over(), "Round should be over after taking the last piece");
        
        // 次のラウンドを開始
        manager.start_next_round();
        
        // ラウンド番号をチェック
        assert_eq!(manager.session.round, 2);
    }


    #[test]
    fn test_invalid_moves() {
        // GameManagerの初期化
        let mut manager = GameManager::new(BoardSize::Small);
        
        // イベントレコーダーを直接GameManagerに組み込まずにテスト
        
        // ゲーム開始
        manager.start_game();
        
        // 無効な移動（クロスチップと同じ位置）
        let cross_pos = manager.session.board.cross_position;
        let result1 = manager.session.process_move(cross_pos);
        
        // 結果をチェック
        assert!(result1.is_err(), "Move to cross position should fail");
        
        // 無効な移動（盤面外）
        let result2 = manager.session.process_move((10, 10));
        
        // 結果をチェック
        assert!(result2.is_err(), "Move outside board should fail");
    }

    #[test]
    fn test_score_calculation() {
        // GameManagerの初期化
        let mut manager = GameManager::new(BoardSize::Small);
        
        // ゲーム開始
        manager.start_game();
        
        // 有効な移動先を取得
        let valid_moves = manager.session.board.get_valid_moves(Player::First);
        let move_target = valid_moves[0];
        
        // 移動前のスコアを記録
        let before_score = manager.session.scores.get(&Player::First).unwrap().total;
        
        // 移動先の駒の値を取得
        let piece_value = match manager.session.board.get_piece(move_target.0, move_target.1) {
            Piece::Number(val) => val,
            _ => 0,
        };
        
        // 移動を実行
        manager.make_move(move_target);
        
        // 移動後のスコアをチェック
        let after_score = manager.session.scores.get(&Player::First).unwrap().total;
        assert_eq!(after_score - before_score, piece_value);
    }
}
