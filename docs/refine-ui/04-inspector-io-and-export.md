# Step 4: 右インスペクタの I/O と Export 統合

## 目的

`Graph Input` と `Export Image` を右パネルの `I/O` タブへ集約し，浮動ウィンドウ依存を外す．

## 対象ファイル

- `src/components/inspector_panel.rs`
- `src/components/graph_io.rs`
- `src/app.rs`

## 実装内容

### 1. I/O タブに Graph Text セクションを作る

表示内容:

- `code_editor`
- `Copy`
- `Apply`

### 2. 自動同期条件を更新する

現状:

```rust
if !app.ui.cursor_hover.get_input_window() {
    app.ui.input_text = app.state.graph.encode(app.state.zero_indexed);
}
```

移行後:

```rust
if !app.ui.input_has_focus {
    app.ui.input_text = app.state.graph.encode(app.state.zero_indexed);
}
```

### 3. `Copy` / `Apply` を移植する

- `Copy` は `ctx.copy_text(app.ui.input_text.clone())`
- `Apply` は既存 `draw_graph_io` の parse/rebuild/reset の流れを維持する

維持する処理:

- `BaseGraph::parse`
- `rebuild_from_basegraph`
- `graph_view.reset_for_graph`
- `next_z_index` の更新
- `is_animated = true`
- エラー時の `error_message` 更新

### 4. Export UI を I/O タブへ移植する

表示内容:

- `Format: PNG / SVG`
- `Export`

利用 API:

- `app.export.format()`
- `app.export.set_format(format)`
- `app.request_export_image(ctx)`

## 実装メモ

- `code_editor` の `has_focus()` から `input_has_focus` を更新する
- フォーカス中はショートカット抑止も維持する
- `I/O` タブは縦長になりやすいので `ScrollArea` 併用も検討する

## 完了条件

- 浮動 `Graph Input` を使わなくても入出力作業が完結する
- フォーカス中に入力内容が勝手に上書きされない
- PNG / SVG の切替と Export が右パネルから動く
