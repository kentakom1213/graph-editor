# Step 1: 状態とレイアウト土台の整理

## 目的

固定パネル構成へ移行するために，UI 状態とコンポーネント境界を先に整理する．
この段階では，既存機能の移植先を定義し，後続ステップが安全に進められる状態を作る．

## 対象ファイル

- `src/app.rs`
- `src/state/mod.rs`
- `src/components/mod.rs`
- `src/components/top_panel.rs`

## 実装内容

### 1. Inspector 用のタブ状態を追加する

- `InspectorTab` enum を追加する
- 値は `Graph` / `View` / `Io` の 3 つにする
- `Default` は `Graph` にする

### 2. `UiState` を更新する

- `panel_tab: PanelTabState` を将来的に廃止する
- `inspector_tab: InspectorTab` を追加する
- `input_has_focus: bool` を追加する
- `error_message`，`confirm_clear_all`，`input_text` は維持する

### 3. `CursorHoverState` を新レイアウト向けに整理する

- 追跡対象を `top_panel` / `tool_bar` / `inspector_panel` に寄せる
- `input editor` のフォーカスは `input_has_focus` 側で扱う
- `any()` が「キャンバス以外にいる」を正しく判定できる状態を維持する

### 4. コンポーネント export を更新する

- `tool_bar.rs` と `inspector_panel.rs` を追加できるよう `src/components/mod.rs` を更新する
- 旧 `draw_edit_menu` / `draw_color_settings` / `draw_graph_io` は移行完了まで残してよい

## この段階で決めておくこと

- `PanelTabState` を即時削除するか，移行中だけ残すか
- `CursorHoverState` のフィールド名を全面更新するか，既存 getter/setter を流用するか
- `footer.rs` を最終的に改名するかどうか

## 完了条件

- `UiState` に新レイアウト用の状態が揃っている
- `CursorHoverState::any()` の意味が新レイアウトと一致している
- `app.rs` から新コンポーネントを呼べる前提が整っている
