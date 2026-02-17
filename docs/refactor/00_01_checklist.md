# Refactoring plan checklist

docs/refactor/00_strategy.md の優先度をベースに、実行順のチェックリストをまとめる。

## Phase 1: 描画とデータ更新の分離 (課題 4)
- [ ] 描画ループから破壊的更新処理を分離する方針を決める
- [ ] `restore_graph` の責務を確認し、描画以外の操作フェーズへ移動する
- [ ] `apply_deletions()` 等の明示的なメンテナンス API を定義する
- [ ] 描画は不変スナップショット参照のみにする
- [ ] 既存の動作差分を確認するための最小テスト/手動手順を用意する

## Phase 2: モデル分離と状態分割 (課題 1, 3)
- [ ] `GraphEditorApp` に集中している状態の分割方針を決める
- [ ] `AppState` / `UiState` / `ExportState` の境界を定義する
- [ ] `GraphModel` と `GraphViewState` を分離する設計を作る
- [ ] `Vertex` / `Edge` から UI 依存フィールドを移す
- [ ] `GraphEditorApp` をオーケストレーターとして薄くする

## Phase 3: ID 解決と性能改善 (課題 5)
- [ ] `VertexId` の導入方針を決める (安定 ID)
- [ ] 頂点参照を `HashMap<VertexId, Vertex>` 等で O(1) 化する
- [ ] 辺が `VertexId` を保持する形に変更する
- [ ] 描画時の頂点検索が O(n\*m) にならないことを確認する

## Phase 4: エクスポートと設定の整理 (課題 7, 8)
- [ ] `ExportService` の責務と状態遷移を定義する
- [ ] `GraphEditorApp` からエクスポート処理を分離する
- [ ] `AppConfig` の拡張方法を整理する (列挙型 + パラメータ構造体)
- [ ] UI での切り替えやシリアライズ前提の型設計を固める

## Phase 5: テスト追加 (課題 9)
- [ ] 優先ユースケース (parse / rebuild / 変換) をリスト化する
- [ ] 重要ロジックの単体テストを追加する
- [ ] リファクタリング後の回帰を確認するためのテストを整備する
