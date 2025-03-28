# Micattix

[![CI](https://github.com/MrBearing/micattix/actions/workflows/ci.yml/badge.svg)](https://github.com/MrBearing/micattix/actions/workflows/ci.yml)

Micattixは、MattixボードゲームをプレイするためのエンジンをRustで実装したライブラリです。2人プレイモードと4人プレイモードをサポートし、コンソールUIとggezを使用したグラフィカルUIの両方を提供します。名前は雲母(mica)の完全へき開性（一方向のみに割れる性質）にちなんでおり、ゲームの一方向移動という特性を表しています。

## ゲームルール

* 4x4または6x6の盤面を使用
* 2人または4人のプレイヤーが交互にプレイ
* 先攻・3番目のプレイヤーは横軸にのみ移動でき、後攻・4番目のプレイヤーは縦軸にのみ移動できる
* 駒の種類:
  * 4x4の場合: 1～7の数字が各2個、8の数字が1個、クロスチップが1個
  * 6x6の場合: 1～9の数字が各2個、-1～-10と+10の数字が各1個、クロスチップが1個
* クロスチップを移動させて、移動先にある数字の駒を取得
* すべての駒を取得したら、合計点数の高いプレイヤーが勝利

## 特徴

* ゲームロジックとUIの明確な分離
* イベント駆動設計
* 複数のラウンドをサポート
* 2人プレイと4人プレイの両方をサポート
* カスタムUIに対応するインターフェース

## 使用方法

### コンソールUIの実行

```bash
cargo run --bin micattix-console
```

### グラフィカルUIの実行

```bash
cargo run --bin micattix-ggez --features ggez_ui
```

### ライブラリとして使用

```rust
use micattix::core::{Board, BoardSize, GameMode, Player};
use micattix::game::GameManager;

// 新しいゲームを作成（2プレイヤーモード）
let mut manager = GameManager::new(BoardSize::Small, GameMode::TwoPlayers);

// カスタムリスナーを追加
manager.add_listener(Box::new(MyCustomListener::new()));

// ゲーム開始
manager.start_game();

// プレイヤーの移動を処理
manager.make_move((1, 2));

// ラウンド終了後、次のラウンドを開始
if manager.session.is_round_over() {
    manager.start_next_round();
}

// ゲーム終了
manager.end_game();
```

## プロジェクト構造

- `src/core.rs` - ゲームの基本要素(盤面、駒、プレイヤーなど)
- `src/game.rs` - ゲームセッション管理とイベント処理
- `src/ui.rs` - UIの実装とインターフェース
- `src/bin/console.rs` - コンソールUIの実装
- `src/bin/ggez.rs` - ggezを使用したグラフィカルUIの実装

## カスタムUIの作成

カスタムUIを実装するには、`GameEventListener`トレイトを実装します:

```rust
use micattix::game::{GameEvent, GameEventListener};

struct MyCustomUI;

impl GameEventListener for MyCustomUI {
    fn on_event(&mut self, event: GameEvent) {
        match event {
            GameEvent::GameStarted => {
                // ゲーム開始時の処理
            },
            GameEvent::MoveMade(player, target, piece) => {
                // 駒が動いた時の処理
            },
            // その他のイベント
            _ => {},
        }
    }
}
```

## ライセンス

MITライセンスの下で公開されています。
