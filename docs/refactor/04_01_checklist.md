# Phase 4 Checklist: エクスポートと設定の整理

## 1. ExportService の設計
- [ ] `ExportService` の責務と API を決める
- [ ] 状態遷移 (Idle/Requested/Capturing/Writing/Failed) を定義する
- [ ] エラー保持の方式を決める

## 2. GraphEditorApp からの分離
- [ ] エクスポート関連フィールドを `ExportService` に移動する
- [ ] `GraphEditorApp` の呼び出しを `ExportService` 経由に変更する
- [ ] `components/edit_menu.rs` で export 操作を整理する

## 3. AppConfig の拡張整理
- [ ] `VisualizerKind` / `SimulatorKind` を導入する
- [ ] `VisualizerConfig` / `SimulatorConfig` を構造体で整理する
- [ ] `Box<dyn Visualizer>` / `Box<dyn Simulator>` を置き換える

## 4. UI と保存形式の整理
- [ ] UI で `Kind` 切り替えができるようにする
- [ ] 保存/復元で `Kind` を扱う方針を決める
- [ ] 互換性維持の方針を決める

## 5. 動作確認
- [ ] エクスポートの一連動作が従来通り動くこと
- [ ] 設定切り替えが UI で反映されること
- [ ] `cargo check` が通る

## 6. 最小限のファイル分割
- [ ] `src/state/` を追加し `AppState` / `UiState` / `ExportState` を移動する
- [ ] `src/export.rs` の分割方針を決める (service/codec)
- [ ] 分割後の `mod.rs` で公開 API を固定する
