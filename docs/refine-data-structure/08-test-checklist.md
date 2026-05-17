# 変更時の注意点とテスト観点

## 注意点

### `Vertex.id` と `Vec` index の関係

現在の設計では，`reindex_vertices` によって `Vertex.id` を `Vec` の添字と一致させる方針になっている．
この前提を維持するなら，頂点 ID から頂点を得るために `HashMap` を持つ必要は薄い．

ただし，削除フラグが立った状態で `apply_deletions` 前の頂点が残る場合には注意する．
`is_deleted` の頂点を参照しないようにするか，削除処理のタイミングを明確にする．

### `GraphViewState` との同期

`Graph` は頂点・辺の構造を持ち，`GraphViewState` は色・ラベル・選択状態などの見た目を持つ．

高速化のために `Graph` 側で `edge_set` や `adjacency` を持つ場合，`GraphViewState.edges` と `Graph.edges` の添字対応を壊さないようにする必要がある．

特に，以下では対応を確認する．

* `add_vertex`
* `add_edge`
* `apply_deletions`
* `reset_for_graph`
* `rebuild_from_basegraph`
* `clear`

### `is_directed` 切り替え

`is_directed` を切り替えると，`edge_set` のキー正規化と `adjacency` の意味が変わる．
`is_directed` を変更した直後に以下を行う．

```rust
graph.rebuild_edge_set();
graph.rebuild_adjacency();
```

## 基本操作

* 頂点追加ができる
* 辺追加ができる
* 同じ辺を重複追加できない
* 無向グラフで逆向き辺が重複扱いになる
* 有向グラフで逆向き辺を別辺として扱える
* 頂点削除後に辺が正しく消える
* 辺削除後に `edge_set` / `adjacency` が壊れない

## 描画

* 頂点位置が正しく描画される
* 辺が正しく描画される
* 有向辺の矢印が正しく描画される
* 双方向辺が曲線で描画される
* 辺ラベル位置が壊れない
* 頂点ラベル位置が壊れない

## 当たり判定

* 頂点 hover が動く
* 頂点ドラッグが動く
* 辺 hover が動く
* 曲線辺 hover が動く
* 頂点上にマウスがあるとき，辺ではなく頂点が優先される
* Delete / Colorize / Normal モードで挙動が変わらない

## 力学モデル

* アニメーションが動く
* 頂点が NaN にならない
* 辺でつながった頂点が引き合う
* 離れた頂点が反発する
* 最大速度 `max_v` が効く
* `is_directed` の違いで意図しないクラッシュが起きない

## 入出力

* `encode` の結果が変わらない
* `rebuild_from_basegraph` 後に `adjacency` / `edge_set` が正しく構築される
* `clear` 後に補助データも空になる
* `apply_deletions` 後に `Graph` と `GraphViewState` の対応が保たれる

## 性能確認の目安

### 小規模

* 頂点 50，辺 100
* 現状でも十分軽いはず
* 挙動の回帰確認を重視する

### 中規模

* 頂点 300，辺 600
* `adjacency` 導入の効果が出始める
* アニメーション時の差を見る

### 大規模以上

最初から想定しない．
必要になった段階で grid 近似，Barnes-Hut 法，描画 culling などを別途検討する．
