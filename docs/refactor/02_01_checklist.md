# Phase 2 Checklist: モデル分離と状態分割

## 1. 状態分割方針の確定
- [ ] `GraphEditorApp` のフィールドを分類し、移動先を決める
- [ ] `AppState` / `UiState` / `ExportState` の境界を決める
- [ ] `UiState` の保存/復元の扱いを決める

## 2. `GraphModel` / `GraphViewState` 設計
- [ ] `GraphModel` に残すデータ範囲を定義する
- [ ] `GraphViewState` に移す UI 情報を定義する
- [ ] 描画用 snapshot の構築経路を決める

## 3. UI 依存フィールドの移動
- [ ] `Vertex` / `Edge` の UI 依存フィールドを一覧化する
- [ ] `GraphViewState` の保持形式を決める (例: `HashMap<VertexId, VertexView>`)
- [ ] `GraphModel` / `GraphViewState` の同期ルールを決める

## 4. `GraphEditorApp` の薄型化
- [ ] `UiState` を導入し、関連フィールドを移動する
- [ ] `ExportState` を導入し、関連フィールドを移動する
- [ ] `GraphState` / `AppState` を導入し、操作状態を移動する
- [ ] `components/*` の参照経路を新構造に合わせる

## 5. 移行手順と安全策
- [ ] 新 struct を追加し、`GraphEditorApp` から参照可能にする
- [ ] UI 状態 (`panel_tab`, `cursor_hover`, `input_text`) を移行する
- [ ] Export 状態を移行する
- [ ] `GraphViewState` を導入し UI フィールドを段階的に移す
- [ ] `GraphSnapshot` を `GraphModel + GraphViewState` で構築する
- [ ] 変更のたびに `cargo check` を通す

## 完了条件
- [ ] `GraphEditorApp` が状態の直接保持を減らしている
- [ ] `Vertex` / `Edge` から UI 依存フィールドが移動済み
- [ ] 描画/操作が `GraphModel` + `GraphViewState` 構造になっている
- [ ] `cargo check` が通る
