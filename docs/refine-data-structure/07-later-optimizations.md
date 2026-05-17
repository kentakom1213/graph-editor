# 後回しにする最適化

## 頂点描画順のソート差分化

### 目的

毎フレーム `sorted_by_key` を行わないようにする．

### 改善案

`GraphViewState` に描画順を保持する．

```rust
pub struct GraphViewState {
    pub vertices: Vec<VertexViewState>,
    pub edges: Vec<EdgeViewState>,
    pub vertex_draw_order: Vec<usize>,
    pub vertex_draw_order_dirty: bool,
}
```

`z_index` が変わるタイミングで dirty にする．

* 頂点追加時
* 頂点クリック時
* 頂点ドラッグ開始時
* `reset_for_graph` 時
* `apply_deletions` 時

描画前に必要なときだけ再構築する．

```rust
impl GraphViewState {
    pub fn ensure_vertex_draw_order(&mut self) {
        if !self.vertex_draw_order_dirty {
            return;
        }

        self.vertex_draw_order = (0..self.vertices.len()).collect();
        self.vertex_draw_order
            .sort_by_key(|&idx| self.vertices[idx].z_index);

        self.vertex_draw_order_dirty = false;
    }
}
```

この改善の優先度は `adjacency`，`edge_set`，辺 hit-test より低い．
最初は `render_vertices` の `sorted_by_key` はそのままでもよい．

## マウス入力が変化したときだけ当たり判定する

### 目的

マウスが動いていないフレームで，辺・頂点の hover 判定を繰り返さないようにする．

### 追加する状態案

```rust
pub struct UiState {
    pub last_pointer_pos: Option<egui::Pos2>,
    pub hovered_vertex: Option<usize>,
    pub hovered_edge: Option<usize>,
    pub hit_test_dirty: bool,
}
```

### dirty にする条件

* マウス位置が変わった
* グラフ構造が変わった
* 頂点位置が変わった
* affine が変わった
* edit_mode が変わった
* vertex_radius や stroke_width が変わった

最初の実装では，以下の条件だけでもよい．

* マウス位置が変わった
* アニメーション中である
* ドラッグ中である
* グラフが変更された

ただし，egui の `response.hovered()` を使う頂点当たり判定は UI の仕組みに乗っているため，手動キャッシュしすぎると挙動が不自然になる可能性がある．
最初は辺の hit-test だけを対象にするのが安全である．

## 力学モデルの近似

現在の力学モデルでは，各頂点について他のすべての頂点からの反発力を計算している．
この処理が必要になる規模は最初から想定しないため，Step 2 の段階では近似法を入れない．

必要になった段階で，以下を検討する．

* grid 近似
* Barnes-Hut 法
* 遠方頂点のサンプリング
* 反発力計算の間引き
* アニメーション対象頂点の制限
* フレーム時間上限による更新打ち切り

まずデータ構造の改善を行い，実測してから判断する．
