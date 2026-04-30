# Step 3: 右インスペクタの Graph / View タブ実装

## 目的

既存 `draw_edit_menu` のうち，グラフ設定と表示設定を右側インスペクタへ移す．

## 対象ファイル

- `src/components/inspector_panel.rs`
- `src/components/edit_menu.rs`
- `src/state/mod.rs`
- `src/components/mod.rs`

## 実装内容

### 1. `draw_inspector_panel` を新設する

関数シグネチャ:

```rust
pub fn draw_inspector_panel(app: &mut GraphEditorApp, ctx: &egui::Context)
```

パネル構成:

```rust
egui::SidePanel::right("inspector_panel")
    .resizable(true)
    .default_width(260.0)
    .min_width(220.0)
```

上部に以下のタブを置く:

- `Graph`
- `View`
- `I/O`

### 2. Graph タブを実装する

表示セクション:

- `Indexing`
- `Direction`
- `Operations`
- `Danger Zone`

移植対象:

- `0-indexed / 1-indexed`
- `Undirected / Directed`
- `Complement`
- `Revert Edge`
- `Reset Colors`
- `Clear All`

制御条件:

- `Complement` は無向グラフ時のみ有効
- `Revert Edge` は有向グラフ時のみ有効
- `Clear All` は直接実行せず `app.ui.confirm_clear_all = true`

### 3. View タブを実装する

表示セクション:

- `Display`
- `Simulation`

移植対象:

- `app.state.show_number`
- `app.state.is_animated`

## 実装メモ

- `draw_edit_menu` からグラフ操作系を剥がす前提で，処理の重複期間は短く保つ
- 右パネル上でも `cursor_hover` が正しく立つようにする
- セクション見出しを残しておくと後続の設定追加に耐えやすい

## 完了条件

- 右インスペクタにタブ UI がある
- `Graph` タブから既存のグラフ操作が実行できる
- `View` タブから番号表示とアニメーション切替ができる
