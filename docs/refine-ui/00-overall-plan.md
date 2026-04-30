# Graph Editor UI 整理実装指示

## 目的

Graph Editor の UI を，現状の「左メニュー + 浮動ウィンドウ複数」から，次のような役割別レイアウトに整理する．

- 左側: 編集ツールを選ぶための細いツールバー
- 中央: グラフ編集キャンバス
- 右側: グラフ設定・表示設定・入出力をまとめた詳細パネル
- 下部: 現在状態を表示するステータスバー
- 上部: アプリタイトルと主要アクション

現在は `Menu`，`Color`，`Input` の表示切替が上部タブバーにあり，`Color` と `Graph Input` は浮動ウィンドウとして表示されている．これを固定パネル中心の構成に変更する．

関連ファイルの中心は以下．

- `src/app.rs`
- `src/components/top_panel.rs`
- `src/components/edit_menu.rs`
- `src/components/color_panel.rs`
- `src/components/graph_io.rs`
- `src/components/footer.rs`
- `src/components/central_panel.rs`
- `src/components/mod.rs`

## 目標レイアウト

最終的な画面構成は次のイメージにする．

```text
+----------------------------------------------------------------------------------+
| Graph Editor                                      [Export] [Settings]             |
+----------------------------------------------------------------------------------+
| Tools |                                                     | Inspector           |
|       |                                                     |---------------------|
| [N]   |                                                     | [Graph][View][I/O]   |
| [V]   |                                                     |                     |
| [E]   |                                                     | Graph / View / I/O  |
| [C]   |                Graph Canvas                         | の内容を表示        |
| [D]   |                                                     |                     |
|       |                                                     |                     |
|-------|                                                     |                     |
| Color |                                                     |                     |
| [Def] |                                                     |                     |
| [Red] |                                                     |                     |
| [Blu] |                                                     |                     |
| [...] |                                                     |                     |
+-------+-----------------------------------------------------+---------------------+
| Mode: Add Edge | vertices: 8 | edges: 10 | 1-indexed | Directed       vX.Y GitHub |
+----------------------------------------------------------------------------------+
```

## 実装方針

既存の機能は削除せず，配置だけを整理する．

現在の `draw_edit_menu` に入っている機能を，次のように分割する．

```text
Tools:
  - Normal
  - Add Vertex
  - Add Edge
  - Colorize
  - Delete
  - selected color

Graph:
  - 0-indexed / 1-indexed
  - Undirected / Directed
  - Complement
  - Revert Edge
  - Reset Colors
  - Clear All

View:
  - Show Numbers
  - Animate

I/O:
  - Graph Input
  - Copy
  - Apply
  - Export Image
  - PNG / SVG
```

## 追加・変更するコンポーネント

### 1. 左ツールバーを追加する

新しく `src/components/tool_bar.rs` を作成する．

関数名は次にする．

```rust
pub fn draw_tool_bar(app: &mut GraphEditorApp, ctx: &egui::Context)
```

左側に細い `SidePanel` を表示する．

推奨 ID は `"tool_bar"`．

幅はまず固定でよい．

```rust
egui::SidePanel::left("tool_bar")
    .resizable(false)
    .exact_width(72.0)
```

中身は縦方向に配置する．

配置するボタンは以下．

```text
[N] Normal
[V] Add Vertex
[E] Add Edge
[C] Colorize
[D] Delete
```

既存の `EditMode` の切替と同じ動作にする．

クリック時には既存のメソッドを使う．

```rust
app.switch_normal_mode();
app.switch_add_vertex_mode();
app.switch_add_edge_mode();
app.switch_colorize_mode();
app.switch_delete_mode();
```

ボタンは現在のモードが選択中であることが分かる見た目にする．
実装が簡単な場合は `selectable_label` を使ってよい．

色選択もこのツールバーの下部に置く．
既存の `Colors` を使い，次の色を選べるようにする．

```text
Default
Red
Green
Blue
Yellow
Orange
Violet
Pink
Brown
```

色を選んだら，既存の `draw_color_settings` と同様に `app.state.selected_color` を更新し，編集モードを `Colorize` に切り替える．

```rust
if app.state.selected_color != prev_color {
    app.state.edit_mode = EditMode::default_colorize();
}
```

### 2. 右インスペクタパネルを追加する

新しく `src/components/inspector_panel.rs` を作成する．

関数名は次にする．

```rust
pub fn draw_inspector_panel(app: &mut GraphEditorApp, ctx: &egui::Context)
```

右側に `SidePanel` を表示する．

推奨 ID は `"inspector_panel"`．

```rust
egui::SidePanel::right("inspector_panel")
    .resizable(true)
    .default_width(260.0)
    .min_width(220.0)
```

右パネル内部にはタブを置く．

```text
Graph
View
I/O
```

現在の `PanelTabState` は `Menu`，`Color`，`Input` の表示・非表示用なので，これを置き換えるか，新しい状態を追加する．

推奨する状態は次．

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InspectorTab {
    #[default]
    Graph,
    View,
    Io,
}
```

`UiState` に以下を追加する．

```rust
pub inspector_tab: InspectorTab,
```

`PanelTabState` は不要になるため，最終的には削除してよい．
ただし，差分を小さくしたい場合は一時的に残してもよい．

### 3. Graph タブを実装する

`InspectorTab::Graph` のとき，次の要素を表示する．

```text
Indexing
  - 0-indexed
  - 1-indexed

Direction
  - Undirected
  - Directed

Operations
  - Complement
  - Revert Edge
  - Reset Colors

Danger Zone
  - Clear All
```

既存の `draw_edit_menu` にある処理を移植する．

`Complement` は無向グラフのときだけ有効にする．
現在の処理と同じく，`app.state.graph.is_directed` が `false` のときだけ押せるようにする．

`Revert Edge` は有向グラフのときだけ有効にする．
現在の処理と同じく，`app.state.graph.is_directed` が `true` のときだけ押せるようにする．

`Reset Colors` は `app.state.graph_view.reset_colors()` を呼ぶ．

`Clear All` は直接削除せず，現在と同じく確認モーダルを出す．

```rust
app.ui.confirm_clear_all = true;
```

### 4. View タブを実装する

`InspectorTab::View` のとき，次の要素を表示する．

```text
Display
  - Show Numbers

Simulation
  - Animate
```

既存の状態を使う．

```rust
app.state.show_number
app.state.is_animated
```

将来的にレイアウト・見た目設定を追加しやすいように，セクション名だけは分けておく．

### 5. I/O タブを実装する

`InspectorTab::Io` のとき，現在の `Graph Input` と `Export Image` をまとめて表示する．

表示内容は次．

```text
Graph Text
  - code_editor
  - Copy
  - Apply

Export Image
  - Format: PNG / SVG
  - Export
```

`Graph Input` の既存ロジックを移植する．

現在は `draw_graph_io` 内で，入力ウィンドウにカーソルが乗っていない場合に `input_text` を自動更新している．
右パネル化後は，`code_editor` にフォーカスがない場合だけ自動更新するようにするのが望ましい．

簡単には次のような条件でよい．

```rust
if !app.ui.input_has_focus {
    app.ui.input_text = app.state.graph.encode(app.state.zero_indexed);
}
```

必要なら `UiState` に以下を追加する．

```rust
pub input_has_focus: bool,
```

`Copy` は現在と同じく次を呼ぶ．

```rust
ctx.copy_text(app.ui.input_text.clone());
```

`Apply` は現在の `draw_graph_io` と同じ処理を移植する．
`BaseGraph::parse`，`rebuild_from_basegraph`，`graph_view.reset_for_graph`，`next_z_index` 更新，`is_animated = true` の流れは維持する．

`Export Image` は現在の `draw_edit_menu` にある処理を移植する．

```rust
app.request_export_image(ctx);
```

フォーマット選択も現在と同じく `app.export.format()` と `app.export.set_format(format)` を使う．

### 6. 上部パネルを簡素化する

`src/components/top_panel.rs` の `draw_top_panel` を変更する．

現在は `Menu`，`Color`，`Input` の表示トグルを持っているが，整理後は不要にする．

上部には次だけを置く．

```text
Graph Editor                              [Export] [Settings]
```

まず `Settings` はダミーボタンでよい．
`Export` は押したら `app.request_export_image(ctx)` を呼ぶ．

例．

```rust
egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("Graph Editor").strong());

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("Settings").clicked() {
                // TODO: settings panel
            }

            if ui
                .add_enabled(!app.export.is_busy(), egui::Button::new("Export"))
                .clicked()
            {
                app.request_export_image(ctx);
            }
        });
    });
});
```

### 7. 下部ステータスバーを追加する

現在の `draw_footer` は右下固定の `Area` になっている．
これを `TopBottomPanel::bottom` に変更し，ステータスバーとして使う．

表示内容の例．

```text
Mode: Add Edge | 1-indexed | Directed | vertices: 8 | edges: 10        Graph Editor vX.Y GitHub
```

必要な情報は既存の状態から取る．

```rust
app.state.edit_mode
app.state.zero_indexed
app.state.graph.is_directed
app.state.graph.vertices.len()
app.state.graph.edges().len()
```

削除済み頂点・辺を数えないほうがよい場合は，`is_deleted` を除外して数える．

右側には現在と同じバージョン表示と GitHub リンクを置く．

既存の `draw_footer` の名前はそのままでもよいが，役割としては `draw_status_bar` に近い．
余裕があれば `footer.rs` を `status_bar.rs` に改名する．
改名する場合は `src/components/mod.rs` の export も更新する．

### 8. `app.rs` の描画順を変更する

現在の描画順は以下の構成になっている．

```rust
draw_top_panel(self, ctx);
draw_central_panel(self, ctx);

if self.ui.panel_tab.edit_menu {
    draw_edit_menu(self, ctx);
}
if self.ui.panel_tab.color_settings {
    draw_color_settings(self, ctx);
}
if self.ui.panel_tab.graph_io {
    draw_graph_io(self, ctx);
}

draw_footer(self, ctx);
draw_error_modal(self, ctx);
draw_clear_all_modal(self, ctx);
```

これを次の構成に変更する．

```rust
draw_top_panel(self, ctx);
draw_tool_bar(self, ctx);
draw_inspector_panel(self, ctx);
draw_central_panel(self, ctx);
draw_footer(self, ctx);

draw_error_modal(self, ctx);
draw_clear_all_modal(self, ctx);
```

`draw_edit_menu`，`draw_color_settings`，`draw_graph_io` は最終的には呼ばない．
実装後に不要であれば削除する．
ただし，移行中は残してもよい．

### 9. `cursor_hover` の扱いを整理する

現状は，パネル上にカーソルがあるときにキャンバス操作を抑制するため，`CursorHoverState` が使われている．

右パネル・左ツールバー化後は，少なくとも次を追跡できればよい．

```rust
top_panel
tool_bar
inspector_panel
input_editor
```

既存の `color_window`，`menu_window`，`input_window` は役割が変わるため，次のように置き換える．

```rust
#[derive(Default)]
pub struct CursorHoverState {
    top_panel: bool,
    tool_bar: bool,
    inspector_panel: bool,
}
```

ただし，差分を小さくするなら既存フィールドを流用してもよい．
その場合は次の対応にする．

```text
menu_window  -> tool_bar
color_window -> inspector_panel
input_window -> input editor focus
```

重要なのは，キャンバス上ではないクリックで頂点が追加されないこと．
`add_vertex` は現在と同じく `!app.ui.cursor_hover.any()` を条件にしているため，新しいパネルでも `cursor_hover` を正しく更新する．

### 10. キーボードショートカットは維持する

以下のショートカットは維持する．

```text
Esc       Normal
V         Add Vertex
E         Add Edge
C         Colorize
D         Delete
Shift + D Directed / Undirected 切替
1         0-indexed / 1-indexed 切替
A         Animate 切替
```

`Graph Input` の `code_editor` にフォーカスがある場合は，現在と同様にショートカットを発火させない．

### 11. 既存機能の維持条件

次の機能は，UI 変更後も動作すること．

- 頂点追加
- 頂点ドラッグ
- 辺追加
- 頂点削除
- 辺削除
- 頂点色変更
- 辺色変更
- `0-indexed` / `1-indexed` 切替
- `Directed` / `Undirected` 切替
- 頂点番号表示切替
- アニメーション切替
- グラフ入力の `Copy`
- グラフ入力の `Apply`
- `Complement`
- `Revert Edge`
- `Reset Colors`
- `Clear All`
- `PNG` / `SVG` エクスポート
- エラーモーダル表示
- Clear All 確認モーダル表示

## 推奨する実装順

### Step 1: 状態を追加する

`InspectorTab` を追加し，`UiState` に `inspector_tab` を追加する．

### Step 2: `draw_tool_bar` を作る

`draw_edit_menu` のうち，編集モードと色選択だけを移植する．

### Step 3: `draw_inspector_panel` を作る

まず `Graph` タブだけを実装する．
その後，`View`，`I/O` の順で実装する．

### Step 4: `app.rs` の描画順を変更する

新しい `draw_tool_bar` と `draw_inspector_panel` を呼び出す．
古い `draw_edit_menu`，`draw_color_settings`，`draw_graph_io` の呼び出しを止める．

### Step 5: 上部パネルを簡素化する

`Menu`，`Color`，`Input` のトグルを削除し，タイトルバーにする．

### Step 6: フッターをステータスバー化する

右下 `Area` ではなく，下部 `TopBottomPanel` にする．

### Step 7: 不要コードを削除する

動作確認後，未使用になったコンポーネントを削除する．

候補．

```text
draw_edit_menu
draw_color_settings
draw_graph_io
PanelTabState
```

ただし，機能移植が完了してから削除すること．

## 注意点

`egui::SidePanel` の配置順に注意する．
左ツールバーと右インスペクタを先に描画し，最後に `CentralPanel` を描画すると，中央キャンバスが残り領域を使う．

推奨順は次．

```rust
draw_top_panel(self, ctx);
draw_tool_bar(self, ctx);
draw_inspector_panel(self, ctx);
draw_central_panel(self, ctx);
draw_footer(self, ctx);
```

`Graph Input` の `code_editor` にフォーカスがある間は，`input_text` を自動更新しないこと．
そうしないと，ユーザーが入力中の文字列が現在のグラフ文字列で上書きされる．

`Clear All` はボタン押下で即時実行しないこと．
必ず既存の確認モーダルを経由すること．

`Complement` と `Revert Edge` は有効条件が逆なので注意する．

```text
Complement: Undirected のときのみ有効
Revert Edge: Directed のときのみ有効
```

## 完了条件

以下を満たしたら完了とする．

- 起動時に，左ツールバー，中央キャンバス，右インスペクタ，下部ステータスバーが表示される
- `Color` と `Graph Input` の浮動ウィンドウが表示されない
- 上部の `Menu`，`Color`，`Input` トグルが消えている
- 既存の主要機能がすべて新 UI 上で操作できる
- `cargo fmt` が通る
- `cargo clippy` または既存 CI の `cargo check` が通る
