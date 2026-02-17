# Phase 1 Checklist: 描画とデータ更新の分離

## 1. 破壊的更新の分離方針
- [ ] `restore_graph` の責務分割案を整理する
- [ ] `apply_deletions()` の呼び出しタイミングを決定する
- [ ] `encode()` の破壊的更新をやめる方針を決める

## 2. `restore_graph` の分割・移行
- [ ] `Graph::apply_deletions()` を定義する
- [ ] `Graph::reindex_vertices()` を定義する
- [ ] `restore_graph` を削除またはラッパー化する
- [ ] 描画経路から `restore_graph()` 呼び出しを排除する

## 3. 描画のスナップショット化
- [ ] `GraphSnapshot` / `GraphRenderView` の設計を決める
- [ ] `Graph::snapshot()` の追加を検討する
- [ ] `draw_edges()` / `draw_vertices()` を snapshot 参照に切り替える
- [ ] クリック/ドラッグ等の操作ロジックを描画処理から分離する

## 4. 明示的メンテナンス API の配置
- [ ] `apply_deletions()` を `GraphEditorApp::update()` などに配置する
- [ ] 削除イベント直後にメンテナンスが必要か判断する
- [ ] export/encode 時のメンテナンス呼び出し位置を整理する

## 5. 最小テスト/手動手順
- [ ] 手動確認手順を用意する
  - [ ] 頂点削除で辺が消えること
  - [ ] 連続削除後に描画が崩れないこと
  - [ ] `encode()` / export で削除済み要素が含まれないこと
  - [ ] 削除後のドラッグ/色変更/辺追加が動作すること
- [ ] `Graph::apply_deletions()` のユニットテスト案を検討する

## 完了条件
- [ ] 描画関数は破壊的更新 API を呼ばない
- [ ] `apply_deletions()` が明示的なタイミングで呼ばれている
- [ ] 手動/テスト手順で Phase 1 の差分確認ができる
