#[cfg(test)]
mod integration_tests {
    use micattix::core::{Board, BoardSize, GameMode, Piece, Player};
    use micattix::game::{GameEvent, GameEventListener, GameManager};
    use std::collections::VecDeque;

    // シンプルなイベントリスナー
    struct SimpleEventRecorder {
        events: VecDeque<GameEvent>,
    }

    impl SimpleEventRecorder {
        fn new() -> Self {
            Self {
                events: VecDeque::new(),
            }
        }
    }

    impl GameEventListener for SimpleEventRecorder {
        fn on_event(&mut self, event: GameEvent) {
            self.events.push_back(event);
        }
    }

    #[test]
    fn test_two_player_game_flow() {
        // GameManagerの初期化（2プレイヤーモード）
        let mut manager = GameManager::new(BoardSize::Small, GameMode::TwoPlayers);

        // イベントレコーダーを追加
        let _recorder = SimpleEventRecorder::new();

        // ゲーム開始
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
    fn test_four_player_game_flow() {
        // GameManagerの初期化（4プレイヤーモード）
        let mut manager = GameManager::new(BoardSize::Small, GameMode::FourPlayers);

        // ゲーム開始
        manager.start_game();

        // プレイヤー順番をチェック
        assert_eq!(manager.session.current_player, Player::First);

        // 各プレイヤーの移動をシミュレート
        for expected_player in [Player::First, Player::Second, Player::Third, Player::Fourth] {
            assert_eq!(manager.session.current_player, expected_player);

            // 有効な移動先を取得
            let valid_moves = manager.session.board.get_valid_moves(expected_player);
            if valid_moves.is_empty() {
                // 有効な移動がない場合はスキップ
                continue;
            }

            // 移動を実行
            let move_target = valid_moves[0];
            manager.make_move(move_target);
        }

        // 一巡後は最初のプレイヤーに戻る
        assert_eq!(manager.session.current_player, Player::First);
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

        // ゲームマネージャーを初期化（2プレイヤーモードで）
        let mut manager = GameManager::new_with_board(board, GameMode::TwoPlayers);

        // 先手（横移動）に設定
        manager.session.current_player = Player::First;

        // ゲーム開始
        manager.start_game();

        // 有効な移動先を確認
        let valid_moves = manager
            .session
            .board
            .get_valid_moves(manager.session.current_player);

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
        assert!(
            manager.session.is_round_over(),
            "Round should be over after taking the last piece"
        );

        // 次のラウンドを開始
        manager.start_next_round();

        // ラウンド番号をチェック
        assert_eq!(manager.session.round, 2);
    }

    #[test]
    fn test_four_player_score_calculation() {
        // 4プレイヤーモードでGameManagerを初期化
        let mut manager = GameManager::new(BoardSize::Small, GameMode::FourPlayers);

        // ゲーム開始
        manager.start_game();

        // すべてのプレイヤーが1回ずつ移動
        for player in [Player::First, Player::Second, Player::Third, Player::Fourth] {
            assert_eq!(manager.session.current_player, player);

            // 有効な移動先を取得
            let valid_moves = manager.session.board.get_valid_moves(player);
            if valid_moves.is_empty() {
                continue; // 有効な移動がない場合はスキップ
            }

            // 移動前のスコアを記録
            let before_score = manager.session.scores.get(&player).unwrap().total;

            // 移動先の駒の値を取得
            let target = valid_moves[0];
            let piece_value = match manager.session.board.get_piece(target.0, target.1) {
                Piece::Number(val) => val,
                _ => 0,
            };

            // 移動を実行
            manager.make_move(target);

            // 移動後のスコアをチェック（数値の駒だった場合）
            if piece_value != 0 {
                let after_score = manager.session.scores.get(&player).unwrap().total;
                assert_eq!(after_score - before_score, piece_value);
            }
        }
    }

    #[test]
    fn test_invalid_moves() {
        // GameManagerの初期化
        let mut manager = GameManager::new(BoardSize::Small, GameMode::TwoPlayers);

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
        let mut manager = GameManager::new(BoardSize::Small, GameMode::TwoPlayers);

        // ゲーム開始
        manager.start_game();

        // 有効な移動先を取得
        let valid_moves = manager.session.board.get_valid_moves(Player::First);
        let move_target = valid_moves[0];

        // 移動前のスコアを記録
        let before_score = manager.session.scores.get(&Player::First).unwrap().total;

        // 移動先の駒の値を取得
        let piece_value = match manager
            .session
            .board
            .get_piece(move_target.0, move_target.1)
        {
            Piece::Number(val) => val,
            _ => 0,
        };

        // 移動を実行
        manager.make_move(move_target);

        // 移動後のスコアをチェック
        let after_score = manager.session.scores.get(&Player::First).unwrap().total;
        assert_eq!(after_score - before_score, piece_value);
    }

    #[test]
    fn test_game_mode_players() {
        // 2人モードのプレイヤー確認
        let two_player_manager = GameManager::new(BoardSize::Small, GameMode::TwoPlayers);
        assert_eq!(two_player_manager.session.players.len(), 2);
        assert!(two_player_manager.session.players.contains(&Player::First));
        assert!(two_player_manager.session.players.contains(&Player::Second));
        assert!(!two_player_manager.session.players.contains(&Player::Third));
        assert!(!two_player_manager.session.players.contains(&Player::Fourth));

        // 4人モードのプレイヤー確認
        let four_player_manager = GameManager::new(BoardSize::Small, GameMode::FourPlayers);
        assert_eq!(four_player_manager.session.players.len(), 4);
        assert!(four_player_manager.session.players.contains(&Player::First));
        assert!(four_player_manager
            .session
            .players
            .contains(&Player::Second));
        assert!(four_player_manager.session.players.contains(&Player::Third));
        assert!(four_player_manager
            .session
            .players
            .contains(&Player::Fourth));
    }

    #[test]
    fn test_player_direction() {
        // 横向き移動のプレイヤー
        assert_eq!(
            Player::First.direction(),
            micattix::core::MoveDirection::Horizontal
        );
        assert_eq!(
            Player::Third.direction(),
            micattix::core::MoveDirection::Horizontal
        );

        // 縦向き移動のプレイヤー
        assert_eq!(
            Player::Second.direction(),
            micattix::core::MoveDirection::Vertical
        );
        assert_eq!(
            Player::Fourth.direction(),
            micattix::core::MoveDirection::Vertical
        );
    }
}
