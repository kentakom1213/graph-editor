# Step 6: 描画順統合と旧 UI の整理

## 目的

新レイアウトを `app.rs` に統合し，旧ウィンドウ構成から安全に切り替える．

## 対象ファイル

- `src/app.rs`
- `src/components/mod.rs`
- `src/components/edit_menu.rs`
- `src/components/color_panel.rs`
- `src/components/graph_io.rs`

## 実装内容

### 1. `app.rs` の描画順を変更する

新しい順序:

```rust
draw_top_panel(self, ctx);
draw_tool_bar(self, ctx);
draw_inspector_panel(self, ctx);
draw_central_panel(self, ctx);
draw_footer(self, ctx);

draw_error_modal(self, ctx);
draw_clear_all_modal(self, ctx);
```

### 2. 旧 UI 呼び出しを止める

停止対象:

- `draw_edit_menu`
- `draw_graph_io`

### 3. 一時的な共存期間を最短にする

- 先に新 UI で同等機能が揃ってから旧 UI を外す
- 使わなくなった state と import を削る

### 4. 不要コードを整理する

- 旧 hover 名称が残るなら新意味へ揃える
- 未使用 module / export を片付ける

## 実装メモ

- 移行途中で feature duplication が起きやすいので，最終段で未使用コードを確認する
- `handle_export_events` や modal 系の呼び順は維持する

## 完了条件

- `app.rs` が新しい固定パネル構成で描画している
- 旧浮動 UI なしで基本操作が完結する
- 未使用 state / import / export が整理されている
