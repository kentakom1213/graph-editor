# Step 3: `edge_set` の導入

## 目的

`add_unique_edge` の線形探索をなくし，辺の重複判定を高速化する．

## 追加するフィールド

```rust
pub struct Graph {
    pub is_directed: bool,
    pub affine: Rc<RefCell<Affine2D>>,
    pub vertices: Vec<Vertex>,
    pub edges: Vec<Edge>,
    pub adjacency: Vec<Vec<usize>>,
    pub edge_set: HashSet<(usize, usize)>,
}
```

## 辺キーの正規化

無向グラフでは，`u-v` と `v-u` を同一視する．

```rust
fn edge_key(is_directed: bool, from: usize, to: usize) -> (usize, usize) {
    if is_directed || from <= to {
        (from, to)
    } else {
        (to, from)
    }
}
```

## 実装案

```rust
impl Graph {
    pub fn rebuild_edge_set(&mut self) {
        self.edge_set.clear();

        for edge in self.edges.iter().filter(|e| !e.is_deleted) {
            let key = edge_key(self.is_directed, edge.from, edge.to);
            self.edge_set.insert(key);
        }
    }

    pub fn add_unique_edge(&mut self, from: usize, to: usize) -> bool {
        let key = edge_key(self.is_directed, from, to);

        if self.edge_set.contains(&key) {
            return false;
        }

        self.edges.push(Edge::new(from, to));
        self.edge_set.insert(key);

        if self.adjacency.len() < self.vertices.len() {
            self.adjacency.resize_with(self.vertices.len(), Vec::new);
        }

        self.adjacency[from].push(to);
        if !self.is_directed {
            self.adjacency[to].push(from);
        }

        true
    }
}
```

## 既存コードへの影響

現在は `Graph::add_unique_edge` が static method に近い形で，`is_directed` と `edges` を引数に取っている．
これを `Graph` のメソッドに変更する．

変更前のイメージは次の通りである．

```rust
Graph::add_unique_edge(is_directed, edges_mut, from, to)
```

変更後のイメージは次の通りである．

```rust
graph.add_unique_edge(from, to)
```

ただし，現在の `update_vertex_interactions` では `vertices_mut` と `edges_mut` を同時に借用している可能性がある．
そのままでは `Graph` 全体の mutable borrow が取りにくい場合がある．

その場合は AddEdge 処理をループの外に分離する．

* 頂点ループ中では「追加すべき辺」を一時変数に記録する
* ループ終了後に `graph.add_unique_edge(from, to)` を呼ぶ
* 追加に成功したら `graph_view.add_edge()` を呼ぶ

## 更新が必要なタイミング

以下の処理後に `rebuild_edge_set()` を呼ぶ．

* `apply_deletions`
* `reindex_vertices`
* `rebuild_from_basegraph`
* `clear`
* `is_directed` の切り替え
* 大量の辺を追加した後

## `is_directed` 切り替え時の注意

`is_directed` を切り替えると，`edge_set` のキー正規化と `adjacency` の意味が変わる．
変更直後に以下を行う．

```rust
graph.rebuild_edge_set();
graph.rebuild_adjacency();
```

有向グラフから無向グラフへ切り替えたとき，`u -> v` と `v -> u` が両方存在している場合は無向グラフとして重複辺になる．

対応方針はどちらかを選ぶ．

* 方針 A: 切り替え時に重複辺を削除する
* 方針 B: 内部的には重複辺を残し，描画・`edge_set` では同一視する

Graph Editor の操作感としては方針 A の方が単純である．
ただし，有向グラフとして作った情報が失われるため，切り替え時の挙動として許容できるか確認する．

## 作業内容

1. `Graph` に `edge_set` を追加する
2. `Default` で `edge_set` を初期化する
3. `edge_key` を実装する
4. `rebuild_edge_set` を実装する
5. `add_unique_edge` を `Graph` のメソッドに変更する
6. `update_vertex_interactions` の AddEdge 処理をループ後に辺追加する形に変更する
7. 構造変更後に `edge_set` を再構築する

## 完了条件

* 辺追加時の重複判定が `edges.iter().any(...)` に依存していない
* 無向グラフでは逆向き辺が重複扱いになる
* 有向グラフでは逆向き辺を別辺として扱える
* `Graph.edges` と `GraphViewState.edges` の添字対応が壊れない
