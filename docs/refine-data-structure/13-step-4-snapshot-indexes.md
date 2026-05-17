# Step 4: `GraphSnapshot` の補助インデックス化

## 目的

描画時に毎フレーム `HashMap` や線形探索を繰り返す処理を削減する．

## 追加するフィールド

```rust
pub struct GraphSnapshot {
    pub is_directed: bool,
    pub vertices: Vec<VertexSnapshot>,
    pub edges: Vec<EdgeSnapshot>,
    pub vertex_index: HashMap<usize, usize>,
    pub edge_count: HashMap<(usize, usize), usize>,
}
```

## 補助メソッド

```rust
impl GraphSnapshot {
    pub fn vertex(&self, id: usize) -> Option<&VertexSnapshot> {
        self.vertex_index
            .get(&id)
            .and_then(|&idx| self.vertices.get(idx))
    }

    pub fn vertex_position(&self, id: usize) -> Option<egui::Pos2> {
        self.vertex(id).map(|v| v.position)
    }

    pub fn vertex_radius(&self, id: usize) -> Option<f32> {
        self.vertex(id).map(|v| v.radius)
    }

    pub fn edge_count(&self, from: usize, to: usize) -> usize {
        self.edge_count.get(&(from, to)).copied().unwrap_or(0)
    }
}
```

## snapshot 作成時の構築

```rust
let vertex_index = vertices
    .iter()
    .enumerate()
    .map(|(idx, v)| (v.id, idx))
    .collect();

let mut edge_count = HashMap::new();
for edge in &edges {
    *edge_count.entry((edge.from, edge.to)).or_insert(0) += 1;
    *edge_count.entry((edge.to, edge.from)).or_insert(0) += 1;
}
```

## `render_edges` 側の変更

変更前は，`render_edges` 内で `vertex_positions` と `edge_count` を毎回構築している．
変更後は `GraphSnapshot` の補助メソッドを使う．

```rust
let Some(from_pos) = snapshot.vertex_position(edge.from) else {
    continue;
};
let Some(to_pos) = snapshot.vertex_position(edge.to) else {
    continue;
};

let target_radius = snapshot
    .vertex_radius(edge.to)
    .unwrap_or(config.effective_vertex_radius(snapshot.vertices.len()));
```

## 作業内容

1. `GraphSnapshot` に `vertex_index` を追加する
2. `GraphSnapshot` に `edge_count` を追加する
3. `snapshot` 作成時に `vertex_index` と `edge_count` を構築する
4. `GraphSnapshot::vertex` を追加する
5. `GraphSnapshot::vertex_position` を追加する
6. `GraphSnapshot::vertex_radius` を追加する
7. `GraphSnapshot::edge_count` を追加する
8. `render_edges` の `vertex_positions` ローカル構築を削除する
9. `render_edges` の `edge_count` ローカル構築を削除する
10. 各辺ごとの `snapshot.vertices.iter().find(...)` を削除する

## 完了条件

* `render_edges` が毎フレーム `vertex_positions` を作っていない
* `render_edges` が毎フレーム `edge_count` を作っていない
* 各辺ごとの終点半径取得が線形探索ではない
* 有向辺・双方向辺・辺ラベルの描画が回帰していない
