use ggez::conf::{WindowMode, WindowSetup};
use ggez::event::{self, EventHandler};
use ggez::graphics::{self, Canvas, Color, DrawParam, Text, TextFragment};
use ggez::input::keyboard::KeyInput;
use ggez::input::mouse::MouseButton;
use ggez::mint::Point2;
use ggez::{Context, GameResult};
use micattix::core::{BoardSize, GameMode, Piece, Player};
use micattix::game::{GameEvent, GameEventListener, GameManager};
use std::io::{self, Write};

const CELL_SIZE: f32 = 80.0;
const MARGIN: f32 = 50.0;

struct MicattixGame {
    manager: GameManager,
    selected_cell: Option<(usize, usize)>,
    message: String,
    message_timer: f32,
    round_ending: bool,
    round_end_timer: f32,
}

impl MicattixGame {
    pub fn new(_ctx: &mut Context, size: BoardSize) -> Self {
        // デフォルトで2プレイヤーモードを使用
        let manager = GameManager::new(size, GameMode::TwoPlayers);

        Self {
            manager,
            selected_cell: None,
            message: String::new(),
            message_timer: 0.0,
            round_ending: false,
            round_end_timer: 0.0,
        }
    }

    fn draw_board(&self, canvas: &mut Canvas, ctx: &mut Context) -> GameResult {
        let (rows, cols) = self.manager.session.board.size.dimensions();

        // 背景を描画
        let board_width = cols as f32 * CELL_SIZE;
        let board_height = rows as f32 * CELL_SIZE;

        let board_rect = graphics::Rect::new(MARGIN, MARGIN, board_width, board_height);

        let board_mesh = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            board_rect,
            Color::from_rgb(230, 230, 230),
        )?;

        canvas.draw(&board_mesh, DrawParam::default());

        // セルとその内容を描画
        for row in 0..rows {
            for col in 0..cols {
                // セルの位置を計算
                let x = MARGIN + col as f32 * CELL_SIZE;
                let y = MARGIN + row as f32 * CELL_SIZE;

                // セルの枠を描画
                let cell_rect = graphics::Rect::new(x, y, CELL_SIZE, CELL_SIZE);
                let cell_mesh = graphics::Mesh::new_rectangle(
                    ctx,
                    graphics::DrawMode::stroke(2.0),
                    cell_rect,
                    Color::BLACK,
                )?;

                canvas.draw(&cell_mesh, DrawParam::default());

                // セルの内容を描画
                let piece = self.manager.session.board.get_piece(row, col);
                match piece {
                    Piece::Number(n) => {
                        let text = Text::new(TextFragment::new(n.to_string()).scale(32.0));
                        let text_pos = Point2 {
                            x: x + CELL_SIZE / 2.0 - 10.0,
                            y: y + CELL_SIZE / 2.0 - 16.0,
                        };

                        let color = if n < 0 { Color::RED } else { Color::BLACK };

                        canvas.draw(&text, DrawParam::default().dest(text_pos).color(color));
                    }
                    Piece::Cross => {
                        let text = Text::new(TextFragment::new("X").scale(32.0));
                        let text_pos = Point2 {
                            x: x + CELL_SIZE / 2.0 - 10.0,
                            y: y + CELL_SIZE / 2.0 - 16.0,
                        };

                        canvas.draw(&text, DrawParam::default().dest(text_pos).color(Color::RED));
                    }
                    Piece::Empty => {}
                }
            }
        }

        // 有効な移動先をハイライト
        let current_player = self.manager.session.current_player;
        let valid_moves = self.manager.session.board.get_valid_moves(current_player);

        for (row, col) in valid_moves {
            let x = MARGIN + col as f32 * CELL_SIZE;
            let y = MARGIN + row as f32 * CELL_SIZE;

            let highlight_rect = graphics::Rect::new(x, y, CELL_SIZE, CELL_SIZE);
            let highlight_mesh = graphics::Mesh::new_rectangle(
                ctx,
                graphics::DrawMode::fill(),
                highlight_rect,
                Color::from_rgba(0, 255, 0, 100),
            )?;

            canvas.draw(&highlight_mesh, DrawParam::default());
        }

        // 選択されたセルをハイライト
        if let Some((row, col)) = self.selected_cell {
            let x = MARGIN + col as f32 * CELL_SIZE;
            let y = MARGIN + row as f32 * CELL_SIZE;

            let select_rect = graphics::Rect::new(x, y, CELL_SIZE, CELL_SIZE);
            let select_mesh = graphics::Mesh::new_rectangle(
                ctx,
                graphics::DrawMode::fill(),
                select_rect,
                Color::from_rgba(255, 255, 0, 100),
            )?;

            canvas.draw(&select_mesh, DrawParam::default());
        }

        Ok(())
    }

    fn draw_info(&self, canvas: &mut Canvas, _ctx: &mut Context) -> GameResult {
        // 現在のプレイヤー情報
        let current_player = self.manager.session.current_player;
        let player_text = match current_player {
            Player::First => {
                Text::new(TextFragment::new("Current Player: First (Horizontal)").scale(24.0))
            }
            Player::Second => {
                Text::new(TextFragment::new("Current Player: Second (Vertical)").scale(24.0))
            }
            Player::Third => {
                Text::new(TextFragment::new("Current Player: Third (Horizontal)").scale(24.0))
            }
            Player::Fourth => {
                Text::new(TextFragment::new("Current Player: Fourth (Vertical)").scale(24.0))
            }
        };
        let player_pos = Point2 { x: MARGIN, y: 20.0 };

        canvas.draw(&player_text, DrawParam::default().dest(player_pos));

        // スコア情報
        let first_score = &self.manager.session.scores[&Player::First];
        let second_score = &self.manager.session.scores[&Player::Second];

        // ゲームモードに応じたスコア表示
        let score_text = if self.manager.session.game_mode == GameMode::TwoPlayers {
            Text::new(
                TextFragment::new(format!(
                    "Scores - First: {} | Second: {}",
                    first_score.total, second_score.total
                ))
                .scale(20.0),
            )
        } else {
            // 4人プレイモードの場合
            let third_score = &self.manager.session.scores[&Player::Third];
            let fourth_score = &self.manager.session.scores[&Player::Fourth];
            Text::new(
                TextFragment::new(format!(
                    "Scores - First: {} | Second: {} | Third: {} | Fourth: {}",
                    first_score.total, second_score.total, third_score.total, fourth_score.total
                ))
                .scale(20.0),
            )
        };
        let score_pos = Point2 {
            x: MARGIN,
            y: MARGIN * 2.0 + self.manager.session.board.size.dimensions().0 as f32 * CELL_SIZE,
        };

        canvas.draw(&score_text, DrawParam::default().dest(score_pos));

        // メッセージ
        if self.message_timer > 0.0 {
            let message_text = Text::new(TextFragment::new(&self.message).scale(24.0));
            let message_pos = Point2 {
                x: MARGIN,
                y: MARGIN * 2.5 + self.manager.session.board.size.dimensions().0 as f32 * CELL_SIZE,
            };

            canvas.draw(
                &message_text,
                DrawParam::default().dest(message_pos).color(Color::RED),
            );
        }

        // ラウンド情報
        let round_text = Text::new(
            TextFragment::new(format!("Round: {}", self.manager.session.round)).scale(24.0),
        );
        let round_pos = Point2 {
            x: MARGIN + 400.0,
            y: 20.0,
        };

        canvas.draw(&round_text, DrawParam::default().dest(round_pos));

        // 合計スコア情報
        let total_first = self.manager.session.total_scores[&Player::First];
        let total_second = self.manager.session.total_scores[&Player::Second];

        // ゲームモードに応じた合計スコア表示
        let total_text = if self.manager.session.game_mode == GameMode::TwoPlayers {
            Text::new(
                TextFragment::new(format!(
                    "Total Scores - First: {} | Second: {}",
                    total_first, total_second
                ))
                .scale(20.0),
            )
        } else {
            // 4人プレイモードの場合
            let total_third = self.manager.session.total_scores[&Player::Third];
            let total_fourth = self.manager.session.total_scores[&Player::Fourth];
            Text::new(
                TextFragment::new(format!(
                    "Total Scores - First: {} | Second: {} | Third: {} | Fourth: {}",
                    total_first, total_second, total_third, total_fourth
                ))
                .scale(20.0),
            )
        };
        let total_pos = Point2 {
            x: MARGIN,
            y: MARGIN * 3.0 + self.manager.session.board.size.dimensions().0 as f32 * CELL_SIZE,
        };

        canvas.draw(&total_text, DrawParam::default().dest(total_pos));

        // ゲーム説明
        let help_text = Text::new(
            TextFragment::new("Click on highlighted cells to move. ESC to quit. N for new round.")
                .scale(18.0),
        );
        let help_pos = Point2 {
            x: MARGIN,
            y: MARGIN * 3.5 + self.manager.session.board.size.dimensions().0 as f32 * CELL_SIZE,
        };

        canvas.draw(&help_text, DrawParam::default().dest(help_pos));

        Ok(())
    }

    fn handle_click(&mut self, x: f32, y: f32) {
        // ラウンド終了処理中は操作を受け付けない
        if self.round_ending {
            return;
        }

        // クリック位置がボード上かチェック
        if x < MARGIN || y < MARGIN {
            return;
        }

        let (rows, cols) = self.manager.session.board.size.dimensions();
        let board_width = cols as f32 * CELL_SIZE;
        let board_height = rows as f32 * CELL_SIZE;

        if x > MARGIN + board_width || y > MARGIN + board_height {
            return;
        }

        // セル位置を計算
        let col = ((x - MARGIN) / CELL_SIZE) as usize;
        let row = ((y - MARGIN) / CELL_SIZE) as usize;

        if row >= rows || col >= cols {
            return;
        }

        // 有効な移動先かチェック
        let current_player = self.manager.session.current_player;
        let valid_moves = self.manager.session.board.get_valid_moves(current_player);

        if valid_moves.contains(&(row, col)) {
            // 移動を実行
            self.manager.make_move((row, col));
            self.selected_cell = None;

            // ラウンド終了チェック
            if self.manager.session.is_round_over() {
                self.round_ending = true;
                self.round_end_timer = 3.0;
            }
        } else {
            self.selected_cell = Some((row, col));
            self.message = "Invalid move! Select a highlighted cell.".to_string();
            self.message_timer = 2.0;
        }
    }

    fn start_next_round(&mut self) {
        self.manager.start_next_round();
        self.round_ending = false;
        self.message = "New round started!".to_string();
        self.message_timer = 2.0;
    }
}

impl GameEventListener for MicattixGame {
    fn on_event(&mut self, event: GameEvent) {
        match event {
            GameEvent::GameStarted => {
                self.message = "Game started!".to_string();
                self.message_timer = 3.0;
            }
            GameEvent::RoundStarted(round) => {
                self.message = format!("Round {} started!", round);
                self.message_timer = 3.0;
            }
            GameEvent::MoveMade(player, target, piece) => {
                if let Piece::Number(value) = piece {
                    self.message = format!(
                        "{:?} moved to {:?} and got {} points",
                        player, target, value
                    );
                } else {
                    self.message = format!("{:?} moved to {:?}", player, target);
                }
                self.message_timer = 2.0;
            }
            GameEvent::InvalidMove(_player, _target, reason) => {
                self.message = format!("Invalid move: {}", reason);
                self.message_timer = 2.0;
            }
            GameEvent::RoundEnded(winner, _scores) => {
                match winner {
                    Some(w) => self.message = format!("Round ended! Winner: {:?}", w),
                    None => self.message = "Round ended in a draw!".to_string(),
                }
                self.message_timer = 5.0;
            }
            GameEvent::GameEnded(winner, _scores) => {
                match winner {
                    Some(w) => self.message = format!("Game ended! Overall winner: {:?}", w),
                    None => self.message = "Game ended in a draw!".to_string(),
                }
                self.message_timer = 10.0;
            }
        }
    }
}

impl EventHandler for MicattixGame {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        // メッセージタイマーを更新
        let dt = ctx.time.delta().as_secs_f32();
        if self.message_timer > 0.0 {
            self.message_timer -= dt;
        }

        // ラウンド終了タイマーを更新
        if self.round_ending {
            self.round_end_timer -= dt;
            if self.round_end_timer <= 0.0 {
                // 自動的に次のラウンドを開始
                self.start_next_round();
            }
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = Canvas::from_frame(ctx, Color::WHITE);

        self.draw_board(&mut canvas, ctx)?;
        self.draw_info(&mut canvas, ctx)?;

        canvas.finish(ctx)?;
        Ok(())
    }

    fn mouse_button_down_event(
        &mut self,
        _ctx: &mut Context,
        button: MouseButton,
        x: f32,
        y: f32,
    ) -> GameResult {
        if button == MouseButton::Left {
            self.handle_click(x, y);
        }
        Ok(())
    }

    fn key_down_event(&mut self, ctx: &mut Context, input: KeyInput, _repeat: bool) -> GameResult {
        match input.keycode {
            Some(ggez::input::keyboard::KeyCode::Escape) => {
                // ゲーム終了
                self.manager.end_game();
                ctx.request_quit();
            }
            Some(ggez::input::keyboard::KeyCode::N) => {
                // 新しいラウンドを開始（現在のラウンドが終了している場合のみ）
                if self.manager.session.is_round_over() {
                    self.start_next_round();
                } else {
                    self.message =
                        "Cannot start new round until current round is finished!".to_string();
                    self.message_timer = 2.0;
                }
            }
            _ => {}
        }
        Ok(())
    }
}

fn main() -> GameResult {
    println!("Welcome to Micattix!");
    println!("Select board size:");
    println!("1: 4x4");
    println!("2: 6x6");

    let mut input = String::new();
    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut input).unwrap();

    let size = match input.trim() {
        "1" => BoardSize::Small,
        "2" => BoardSize::Large,
        _ => {
            println!("Invalid selection, using 4x4 board");
            BoardSize::Small
        }
    };

    println!("Select game mode:");
    println!("1: 2 Players");
    println!("2: 4 Players");

    let mut input = String::new();
    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut input).unwrap();

    let game_mode = match input.trim() {
        "1" => GameMode::TwoPlayers,
        "2" => GameMode::FourPlayers,
        _ => {
            println!("Invalid selection, using 2 Players mode");
            GameMode::TwoPlayers
        }
    };

    let window_title = match size {
        BoardSize::Small => "Micattix - 4x4",
        BoardSize::Large => "Micattix - 6x6",
    };

    // ウィンドウサイズをボードサイズに応じて調整
    let (rows, cols) = size.dimensions();
    let window_width = MARGIN * 2.0 + cols as f32 * CELL_SIZE;
    let window_height = MARGIN * 4.0 + rows as f32 * CELL_SIZE;

    let cb = ggez::ContextBuilder::new("micattix", "micattix-author")
        .window_setup(WindowSetup::default().title(window_title))
        .window_mode(WindowMode::default().dimensions(window_width, window_height));

    // 音声エラーを無視する - ゲームでは音声を使用しないため
    println!("注意: 音声関連のエラーはゲームには影響しません。無視して進めてください。");

    let (mut ctx, event_loop) = cb.build()?;

    // ゲームインスタンスを作成
    let mut game = MicattixGame::new(&mut ctx, size);

    // ゲームモードを設定
    game.manager.session.game_mode = game_mode;

    // ゲーム開始
    game.manager.start_game();

    // イベントループを実行
    event::run(ctx, event_loop, game)
}
