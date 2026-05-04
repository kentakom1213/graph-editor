# Step 2: 左ツールバーの実装

## 目的

既存 `draw_edit_menu` から，編集モード切替と色選択だけを独立した左ツールバーへ移す．

## 対象ファイル

- `src/components/tool_bar.rs`
- `src/components/color_panel.rs`
- `src/app.rs`
- `src/components/mod.rs`

## 実装内容

### 1. `draw_tool_bar` を新設する

関数シグネチャ:

```rust
pub fn draw_tool_bar(app: &mut GraphEditorApp, ctx: &egui::Context)
```

パネル構成:

```rust
egui::SidePanel::left("tool_bar")
    .resizable(false)
    .exact_width(182.0)
```

### 2. 編集モード切替を移植する

- `Normal`
- `Add Vertex`
- `Add Edge`
- `Colorize`
- `Delete`

切替時は既存メソッドを使う:

- `app.switch_normal_mode()`
- `app.switch_add_vertex_mode()`
- `app.switch_add_edge_mode()`
- `app.switch_colorize_mode()`
- `app.switch_delete_mode()`

### 3. 選択中モードの視覚表現を付ける

- `selectable_label` または `Button::selected` 相当で実装する
- ラベルは短くしても，ホバー文言や見出しで役割が分かるようにする

### 4. 色選択を下部へ移す

対象色:

- `Default`
- `Red`
- `Green`
- `Blue`
- `Yellow`
- `Orange`
- `Violet`
- `Pink`
- `Brown`

色変更時の条件:

```rust
if app.state.selected_color != prev_color {
    app.state.edit_mode = EditMode::default_colorize();
}
```

## 実装メモ

- 既存 `draw_color_settings` の色更新ロジックを流用する
- ツールバー上にカーソルがある間は `cursor_hover.any()` が `true` になるようにする
- モード名とショートカットを同時に見せる前提で，ある程度の横幅を確保する

## 完了条件

- 左側に固定ツールバーが表示される
- 既存ショートカットと同じ編集モード切替が UI から行える
- 色選択で `selected_color` 更新と `Colorize` モード移行が動作する
