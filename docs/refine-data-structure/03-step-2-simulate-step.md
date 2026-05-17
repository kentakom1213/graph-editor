# Step 2: `simulate_step` の整理

## 目的

アニメーション時のフレーム落ちを減らす．
Step 1 で導入した `neighbor_ids` を使い，力学モデル更新中の余分な探索と `clone` を削る．

## 改善方針

まず行うべき改善は以下である．

* `neighbor_vertices` を `neighbor_ids` に置き換える
* 頂点 `clone` を避ける
* 更新前の位置・速度を一時配列に退避する
* 1 step 内での更新順依存をなくす

この段階では，反発力計算はまだ全頂点対のままでよい．
ただし，隣接頂点取得と `clone` の改善だけでも効果がある．

## 実装イメージ

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

## 作業内容

1. `simulate_step` 内の `neighbor_vertices` 呼び出しを探す
2. 位置と速度を `positions` / `velocities` に退避する
3. 反発力計算を退避配列ベースにする
4. 辺による引力計算を `graph.neighbor_ids(i)` ベースにする
5. `next_positions` / `next_velocities` に計算結果を入れる
6. ループ後に `graph.vertices` へ書き戻す

## 完了条件

* `simulate_step` が `neighbor_vertices` に依存していない
* 1 step 中に更新済み頂点の位置を別頂点の計算が参照しない
* 頂点位置や速度が NaN にならない
* 既存のアニメーション挙動が大きく変わらない
