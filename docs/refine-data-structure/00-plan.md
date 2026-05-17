# Graph Editor アルゴリズム高速化計画

## 目的

本ドキュメントは，Graph Editor の描画・操作・力学モデル更新に関するアルゴリズムおよびデータ構造上のボトルネックを整理し，段階的な高速化方針を定めるものである．

GPU 化や描画バックエンドの大規模変更を行う前に，まず CPU 側のデータ構造と毎フレーム処理を改善する．既存の速度課題メモでも，物理シミュレーションの隣接頂点列挙，毎フレームの探索，エッジのヒットテストなどが主要なボトルネックとして整理されている．

主に以下を改善対象とする．

* 隣接頂点列挙の高速化
* 辺の重複判定の高速化
* 毎フレーム構築している補助データのキャッシュ化
* 辺・頂点の当たり判定の候補削減
* 力学モデルの更新処理の軽量化

## 現状の主要な課題

### 1. `Graph` が隣接リストを持っていない

現在の `Graph` は，主に以下の構造を持つ．

```rust
pub struct Graph {
    pub is_directed: bool,
    pub affine: Rc<RefCell<Affine2D>>,
    pub vertices: Vec<Vertex>,
    pub edges: Vec<Edge>,
}
```

この構造では，ある頂点の隣接頂点を列挙するために，毎回すべての辺を走査する必要がある．さらに，隣接頂点の ID から `Vertex` を得るために `vertices` を探索している箇所がある．

この処理は，力学モデルの更新で特に問題になる．既存メモでも，反発力計算が各頂点に対して全頂点走査であり，隣接頂点列挙が `neighbor_vertices` により全エッジ走査と頂点探索を行っていることが指摘されている．

### 2. `add_unique_edge` が既存辺を線形探索している

現在の `add_unique_edge` は，同じ辺が既に存在するかを `edges.iter().any(...)` で判定している．

手動操作で少数の辺を追加するだけなら大きな問題ではないが，以下の場合には効いてくる．

* 入力から大量の辺を構築する場合
* 補グラフ・ランダムグラフ・テンプレート生成を追加する場合
* URL や保存ファイルから大規模グラフを復元する場合
* AddEdge 操作を繰り返す場合

辺の存在判定は `HashSet` で管理するべきである．

### 3. 描画用補助データを毎フレーム作り直している

描画時には，毎フレーム以下のような補助データを構築している．

* `vertex_positions: HashMap<usize, Pos2>`
* `edge_count: HashMap<(usize, usize), usize>`

また，各辺の描画時に，終点頂点の半径を得るために `snapshot.vertices.iter().find(...)` を行っている．

既存メモでも，毎フレーム `edge_count` を `HashMap` で数え直し，各エッジで頂点探索を行っている点が課題として挙げられている．

### 4. 辺の当たり判定が全辺に対して実行される

エッジのヒットテストでは，マウス位置に対してすべての辺との距離を計算している．

直線辺であればまだ軽いが，有向辺の重複を曲線で描く場合，ベジェ曲線との距離計算が必要になる．この処理を全辺に対して毎フレーム行うと，辺数が増えたときに重くなる．既存メモでも，全エッジに対する線・ベジェ距離計算と，マウス移動時のみ必要な処理が毎フレーム行われている点が課題として整理されている．

### 5. 頂点の `z_index` ソートが毎フレーム発生している

頂点描画時に `sorted_by_key` によって `z_index` 順に頂点を並べている．

頂点の重なり順が変わるのは，主にクリック・ドラッグ・追加時である．毎フレームソートする必要はない．

### 6. 力学モデルの反発力計算が重い

↓ この処理が必要になる規模は想定していないので不要

現在の力学モデルでは，各頂点について他のすべての頂点からの反発力を計算している．これは大きいグラフでは本質的に重い．

ただし，最初から Barnes-Hut 法などを入れるよりも，まず隣接リスト導入，余分な `clone` の削除，位置・速度配列の分離などを先に行う方が安全である．

## Phase 1: `Graph` に隣接リストを追加する

### 目的

隣接頂点の列挙を高速化し，力学モデルや今後のグラフアルゴリズム機能の土台を作る．

### 追加するフィールド案

```rust
pub struct Graph {
    pub is_directed: bool,
    pub affine: Rc<RefCell<Affine2D>>,
    pub vertices: Vec<Vertex>,
    pub edges: Vec<Edge>,

    pub adjacency: Vec<Vec<usize>>,
}
```

### 基本方針

`adjacency[v]` に，頂点 `v` の隣接頂点 ID を保持する．

無向グラフでは両方向に追加する．有向グラフでは，通常の隣接リストとして `from` から `to` のみを追加する．ただし，力学モデルで無向的な隣接関係が必要な場合は，シミュレーション用には両方向で構築するか，別途 `undirected_adjacency` を持つ．

最初の実装では，差分更新にこだわらず，グラフ変更後に `rebuild_adjacency()` を呼ぶ方針でよい．

### 実装案

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

### 置き換え方針

現在の `neighbor_vertices` は，以下のような用途に分解する．

* 頂点 ID だけが必要な場合は `neighbor_ids` を使う
* `Vertex` 参照が必要な場合は `neighbor_ids` から `vertices[id]` を参照する

`Vertex` の `id` が常に `vertices` の添字と一致する設計であれば，頂点 ID から頂点を得るために `HashMap` を持つ必要は薄い．現在の設計は `reindex_vertices` により ID を再採番しているため，この方針と相性がよい．

### 更新が必要なタイミング

以下の処理後に `rebuild_adjacency()` を呼ぶ．

* `apply_deletions`
* `reindex_vertices`
* `rebuild_from_basegraph`
* `clear`
* `is_directed` の切り替え
* 大量の辺を追加した後

## Phase 2: `edge_set` を追加して辺の存在判定を高速化する

### 目的

`add_unique_edge` の線形探索をなくし，辺の重複判定を高速化する．

### 追加するフィールド案

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

### 辺キーの正規化

無向グラフでは，`u-v` と `v-u` を同一視する必要がある．そのため，`edge_set` に入れるキーは正規化する．

```rust
fn edge_key(is_directed: bool, from: usize, to: usize) -> (usize, usize) {
    if is_directed || from <= to {
        (from, to)
    } else {
        (to, from)
    }
}
```

### 実装案

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

### 既存コードへの影響

現在は `Graph::add_unique_edge` が static method に近い形で，`is_directed` と `edges` を引数に取っている．これを `Graph` のメソッドに変更するのが望ましい．

変更前のイメージは次の通りである．

```rust
Graph::add_unique_edge(is_directed, edges_mut, from, to)
```

変更後のイメージは次の通りである．

```rust
graph.add_unique_edge(from, to)
```

ただし，現在の `update_vertex_interactions` では `vertices_mut` と `edges_mut` を同時に借用しているため，そのままでは `Graph` 全体の mutable borrow が取りにくい可能性がある．

この場合は，AddEdge 処理をループの外に分離する．

* 頂点ループ中では「追加すべき辺」を一時変数に記録する
* ループ終了後に `graph.add_unique_edge(from, to)` を呼ぶ
* 追加に成功したら `graph_view.add_edge()` を呼ぶ

## Phase 3: `GraphSnapshot` に補助インデックスを持たせる

### 目的

描画時に毎フレーム `HashMap` や探索を繰り返す処理を削減する．

### 改善案

`GraphSnapshot` に以下を追加する．

```rust
pub struct GraphSnapshot {
    pub is_directed: bool,
    pub vertices: Vec<VertexSnapshot>,
    pub edges: Vec<EdgeSnapshot>,

    pub vertex_index: HashMap<usize, usize>,
    pub edge_count: HashMap<(usize, usize), usize>,
}
```

補助メソッドを追加する．

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
        self.vertex(id).and_then(|v| v.radius)
    }

    pub fn edge_count(&self, from: usize, to: usize) -> usize {
        self.edge_count.get(&(from, to)).copied().unwrap_or(0)
    }
}
```

`snapshot()` 作成時に `vertex_index` と `edge_count` を構築する．

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

### `render_edges` 側の変更

変更前は，`render_edges` 内で `vertex_positions` と `edge_count` を毎回構築している．

変更後は，`GraphSnapshot` の補助メソッドを使う．

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

### 期待される効果

* `render_edges` の毎フレーム処理が軽くなる
* 各辺ごとの線形探索を削減できる
* 今後，ヒットテストにも同じインデックスを使える

## Phase 4: 辺の当たり判定に粗い候補判定を入れる

### 目的

すべての辺に対して正確な距離計算を行うのを避ける．特に，曲線辺の距離計算を減らす．

### 直線辺の場合

正確な距離計算の前に，線分を含む bounding box を作り，マウスがその近傍にない場合はスキップする．

```rust
fn expanded_edge_rect(
    from: egui::Pos2,
    to: egui::Pos2,
    margin: f32,
) -> egui::Rect {
    egui::Rect::from_two_pos(from, to).expand(margin)
}
```

使用例は次の通りである．

```rust
let threshold = 10.0;
let rect = expanded_edge_rect(from_pos, to_pos, threshold + vertex_radius);

if !rect.contains(mouse_pos) {
    view.is_pressed = false;
    continue;
}
```

### 曲線辺の場合

曲線辺では，`from`，`control`，`to` を含む矩形を作る．

```rust
fn expanded_bezier_rect(
    from: egui::Pos2,
    control: egui::Pos2,
    to: egui::Pos2,
    margin: f32,
) -> egui::Rect {
    egui::Rect::from_min_max(
        egui::pos2(
            from.x.min(control.x).min(to.x),
            from.y.min(control.y).min(to.y),
        ),
        egui::pos2(
            from.x.max(control.x).max(to.x),
            from.y.max(control.y).max(to.y),
        ),
    )
    .expand(margin)
}
```

この判定を通過した場合だけ，`distance_from_edge_bezier` を呼ぶ．

### さらに改善する場合

必要になった段階で，以下を検討する．

* マウス位置が前フレームから変わっていない場合は再計算しない
* `hovered_edge_index` をキャッシュする
* 近傍候補だけを見るための空間インデックスを導入する

ただし，最初の段階では bounding box だけでよい．実装が単純で，バグが入りにくい．

## Phase 5: 頂点描画順のソートを差分化する

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

ただし，この改善の優先度は `adjacency`，`edge_set`，辺 hit-test より低い．最初は `render_vertices` の `sorted_by_key` はそのままでもよい．

## Phase 6: 力学モデルの内部処理を軽量化する

### 目的

アニメーション時のフレーム落ちを減らす．

まず行うべき改善は以下である．

* `neighbor_vertices` を `neighbor_ids` に置き換える
* 頂点 `clone` を避ける
* 更新前の位置・速度を一時配列に退避する
* 1 ステップ内での更新順依存をなくす

### 実装イメージ

```rust
let positions: Vec<_> = graph.vertices.iter().map(|v| v.position).collect();
let velocities: Vec<_> = graph.vertices.iter().map(|v| v.velocity).collect();

let mut next_positions = positions.clone();
let mut next_velocities = velocities.clone();

for i in 0..n {
    let pos_i = positions[i];
    let vel_i = velocities[i];

    let mut force = egui::Vec2::ZERO;

    for j in 0..n {
        if i == j {
            continue;
        }

        let diff = positions[j] - pos_i;
        let dist_sq = diff.length_sq();

        if dist_sq <= DISTANCE_EPS {
            continue;
        }

        let dir = diff.normalized();
        force += -dir * c / dist_sq;
    }

    for j in graph.neighbor_ids(i) {
        let diff = positions[j] - pos_i;
        let dist = diff.length();

        if dist <= DISTANCE_EPS {
            continue;
        }

        let dir = diff / dist;
        force += dir * (dist - l) * k;
    }

    let mut next_velocity = (vel_i + force * dt / m) * h;

    if next_velocity.length() > max_v {
        next_velocity = next_velocity.normalized() * max_v;
    }

    next_velocities[i] = next_velocity;
    next_positions[i] = pos_i + next_velocity * dt;
}

for i in 0..n {
    graph.vertices[i].velocity = next_velocities[i];
    graph.vertices[i].position = next_positions[i];
}
```

この段階では，反発力計算はまだ全頂点対である．ただし，隣接頂点取得と `clone` の改善だけでも効果がある．

### さらに大規模グラフ向けの改善

必要になった段階で，以下を検討する．

* grid 近似
* Barnes-Hut 法
* 遠方頂点のサンプリング
* 反発力計算の間引き
* アニメーション対象頂点の制限
* フレーム時間上限による更新打ち切り

最初から複雑な近似法を入れる必要はない．まずデータ構造の改善を行い，実測してから判断する．

## Phase 7: マウス入力が変化したときだけ当たり判定する

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

ただし，egui の `response.hovered()` を使う頂点当たり判定は UI の仕組みに乗っているため，手動キャッシュしすぎると挙動が不自然になる可能性がある．最初は辺の hit-test だけを対象にするのが安全である．

## 実装優先度

### 最優先

1. `Graph` に `adjacency` を追加する
2. `neighbor_vertices` を `neighbor_ids` に置き換える
3. `simulate_step` で `neighbor_ids` を使う
4. `apply_deletions` / `reindex_vertices` / `rebuild_from_basegraph` 後に `adjacency` を再構築する

### 優先

5. `edge_set` を追加する
6. `add_unique_edge` を `Graph` のメソッドにする
7. `is_directed` 切り替え時に `edge_set` と `adjacency` を再構築する
8. `GraphSnapshot` に `vertex_index` と `edge_count` を追加する

### 中優先

9. `render_edges` で毎フレーム作っている `HashMap` を削除する
10. `target_radius` の取得を線形探索から `snapshot.vertex_radius` に変える
11. 辺 hit-test に bounding box の早期判定を入れる

### 低優先

12. `z_index` ソートの差分化
13. hit-test 結果のキャッシュ
14. grid / quadtree による空間インデックス
15. Barnes-Hut 法などによる力学モデル近似

## 変更時の注意点

### 1. `Vertex` の `id` と `Vec` の index の関係

現在の設計では，`reindex_vertices` によって `Vertex.id` を `Vec` の添字と一致させる方針になっている．この前提を維持するなら，頂点 ID から頂点を得るために `HashMap` を持つ必要は薄い．

ただし，削除フラグが立った状態で `apply_deletions` 前の頂点が残る場合には注意が必要である．`is_deleted` の頂点を参照しないようにするか，削除処理のタイミングを明確にする．

### 2. `GraphViewState` との同期

`Graph` は頂点・辺の構造を持ち，`GraphViewState` は色・ラベル・選択状態などの見た目を持つ．

高速化のために `Graph` 側で `edge_set` や `adjacency` を持つ場合，`GraphViewState.edges` と `Graph.edges` の添字対応を壊さないようにする必要がある．

特に，以下では対応を確認する．

* `add_vertex`
* `add_edge`
* `apply_deletions`
* `reset_for_graph`
* `rebuild_from_basegraph`
* `clear`

### 3. `is_directed` 切り替え

`is_directed` を切り替えると，`edge_set` のキー正規化と `adjacency` の意味が変わる．

したがって，`is_directed` を変更した直後に以下を行う．

```rust
graph.rebuild_edge_set();
graph.rebuild_adjacency();
```

### 4. 無向グラフで既存の逆向き重複辺がある場合

有向グラフから無向グラフへ切り替えたとき，`u -> v` と `v -> u` が両方存在している場合，無向グラフとしては重複辺になる．

対応方針はどちらかを選ぶ．

方針 A は，切り替え時に重複辺を削除する方法である．内部状態は単純になるが，ユーザーが有向グラフとして作った情報が失われる．

方針 B は，内部的には重複辺を残し，描画・`edge_set` では同一視する方法である．情報は残せるが，実装がやや複雑になる．

Graph Editor の操作感としては，方針 A の方が単純である．ただし，切り替え時の挙動として許容できるかは検討する必要がある．

## 推奨実装手順

### Step 1: `adjacency` のみ導入する

最初の PR では，`edge_set` まで入れず，`adjacency` だけを導入する．差分を小さくし，挙動確認をしやすくするためである．

作業内容は以下の通りである．

* `Graph` に `adjacency` を追加する
* `Default` で `adjacency` を初期化する
* `rebuild_adjacency` を実装する
* `rebuild_from_basegraph` の最後で `rebuild_adjacency` を呼ぶ
* `apply_deletions` / `reindex_vertices` の最後で `rebuild_adjacency` を呼ぶ
* `neighbor_ids` を追加する
* `simulate_step` を `neighbor_ids` に置き換える

### Step 2: `simulate_step` を整理する

* ループ内の `Vertex` clone をなくす
* `positions` / `velocities` の一時配列を使う
* 更新順依存を減らす
* `neighbor_ids` を使って辺による引力を計算する

### Step 3: `edge_set` を導入する

* `Graph` に `edge_set` を追加する
* `edge_key` を実装する
* `rebuild_edge_set` を実装する
* `add_unique_edge` を `Graph` のメソッドに変更する
* `update_vertex_interactions` の AddEdge 処理を，ループ後に辺追加する形に変更する

### Step 4: `GraphSnapshot` を補助インデックス化する

* `GraphSnapshot` に `vertex_index` と `edge_count` を追加する
* `snapshot` 作成時に構築する
* `render_edges` の `vertex_positions` と `edge_count` のローカル構築を削除する
* 各辺ごとの `snapshot.vertices.iter().find` を削除する

### Step 5: edge hit-test に粗判定を入れる

* 直線辺に `expanded_edge_rect` を導入する
* 曲線辺に `expanded_bezier_rect` を導入する
* bounding box 外の辺では距離計算をスキップする
* 挙動確認後，必要なら `hovered_edge` のキャッシュを検討する

## テスト観点

### 基本操作

* 頂点追加ができる
* 辺追加ができる
* 同じ辺を重複追加できない
* 無向グラフで逆向き辺が重複扱いになる
* 有向グラフで逆向き辺を別辺として扱える
* 頂点削除後に辺が正しく消える
* 辺削除後に `edge_set` / `adjacency` が壊れない

### 描画

* 頂点位置が正しく描画される
* 辺が正しく描画される
* 有向辺の矢印が正しく描画される
* 双方向辺が曲線で描画される
* 辺ラベル位置が壊れない
* 頂点ラベル位置が壊れない

### 当たり判定

* 頂点 hover が動く
* 頂点ドラッグが動く
* 辺 hover が動く
* 曲線辺 hover が動く
* 頂点上にマウスがあるとき，辺ではなく頂点が優先される
* Delete / Colorize / Normal モードで挙動が変わらない

### 力学モデル

* アニメーションが動く
* 頂点が NaN にならない
* 辺でつながった頂点が引き合う
* 離れた頂点が反発する
* 最大速度 `max_v` が効く
* `is_directed` の違いで意図しないクラッシュが起きない

### 入出力

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

↓ これ以降は最初から想定していないので不要

### 大規模

* 頂点 1000，辺 3000
* edge hit-test と `render_edges` の改善が効きやすい
* 力学モデルの全頂点対反発はまだ重い可能性がある

### 超大規模

* 頂点 3000 以上
* 現在の力学モデルでは限界が出やすい
* Barnes-Hut 法，grid 近似，描画 culling などを別途検討する

## まとめ

最も効果が出やすい改善は，`Graph` に `adjacency` を持たせることである．これにより，隣接頂点列挙を毎回 `edges` 全体の走査に頼らずに済む．特に，力学モデルの `simulate_step` に対して直接効く．

次に，`edge_set` によって `add_unique_edge` の線形探索をなくす．これにより，辺追加やグラフ構築時のコストを抑えられる．

その後，`GraphSnapshot` に `vertex_index` や `edge_count` を持たせ，描画時に毎フレーム補助 `HashMap` を作る処理を減らす．さらに，辺の当たり判定では bounding box による早期判定を入れ，重い距離計算を必要な候補に限定する．
