# Step 1: `adjacency` の導入

## 目的

隣接頂点の列挙を高速化し，力学モデルや今後のグラフアルゴリズム機能の土台を作る．

## 追加するフィールド

```rust
pub struct Graph {
    pub is_directed: bool,
    pub affine: Rc<RefCell<Affine2D>>,
    pub vertices: Vec<Vertex>,
    pub edges: Vec<Edge>,
    pub adjacency: Vec<Vec<usize>>,
}
```

## 基本方針

`adjacency[v]` に頂点 `v` の隣接頂点 ID を保持する．

無向グラフでは両方向に追加する．
有向グラフでは，通常の隣接リストとして `from` から `to` のみを追加する．
力学モデルで無向的な隣接関係が必要になった場合は，シミュレーション用に両方向で構築するか，別途 `undirected_adjacency` を検討する．

最初の実装では差分更新にこだわらず，グラフ変更後に `rebuild_adjacency()` を呼ぶ．

## 実装案

```rust
impl Graph {
    pub fn rebuild_adjacency(&mut self) {
        let n = self.vertices.len();
        self.adjacency = vec![Vec::new(); n];

        for edge in self.edges.iter().filter(|e| !e.is_deleted) {
            if edge.from >= n || edge.to >= n {
                continue;
            }

            self.adjacency[edge.from].push(edge.to);

            if !self.is_directed {
                self.adjacency[edge.to].push(edge.from);
            }
        }
    }

    pub fn neighbor_ids(&self, id: usize) -> impl Iterator<Item = usize> + '_ {
        self.adjacency
            .get(id)
            .into_iter()
            .flatten()
            .copied()
    }
}
```

## 置き換え方針

現在の `neighbor_vertices` は用途に応じて分解する．

* 頂点 ID だけが必要な場合は `neighbor_ids` を使う
* `Vertex` 参照が必要な場合は `neighbor_ids` から `vertices[id]` を参照する

`Vertex.id` が常に `vertices` の添字と一致する設計であれば，頂点 ID から頂点を得るために `HashMap` を持つ必要は薄い．
現在の設計は `reindex_vertices` により ID を再採番しているため，この方針と相性がよい．

## 更新が必要なタイミング

以下の処理後に `rebuild_adjacency()` を呼ぶ．

* `apply_deletions`
* `reindex_vertices`
* `rebuild_from_basegraph`
* `clear`
* `is_directed` の切り替え
* 大量の辺を追加した後

## 作業内容

1. `Graph` に `adjacency` を追加する
2. `Default` で `adjacency` を初期化する
3. `rebuild_adjacency` を実装する
4. `neighbor_ids` を追加する
5. `rebuild_from_basegraph` の最後で `rebuild_adjacency` を呼ぶ
6. `apply_deletions` / `reindex_vertices` の最後で `rebuild_adjacency` を呼ぶ
7. `clear` 後に `adjacency` も空にする

## 完了条件

* 隣接頂点列挙で全辺走査を使わない経路が用意されている
* 頂点削除・再採番・再構築後に `adjacency` が壊れない
* 既存の頂点追加・辺追加・削除操作が回帰していない
