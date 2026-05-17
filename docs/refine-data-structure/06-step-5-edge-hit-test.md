# Step 5: edge hit-test の粗判定

## 目的

すべての辺に対して正確な距離計算を行うのを避ける．
特に，曲線辺の距離計算を減らす．

## 直線辺の場合

正確な距離計算の前に，線分を含む bounding box を作る．
マウスがその近傍にない場合はスキップする．

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

## 曲線辺の場合

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

## 作業内容

1. 直線辺用に `expanded_edge_rect` を追加する
2. 曲線辺用に `expanded_bezier_rect` を追加する
3. edge hit-test の正確な距離計算前に bounding box 判定を入れる
4. bounding box 外の辺では距離計算をスキップする
5. 曲線辺で `distance_from_edge_bezier` の呼び出し回数が減ることを確認する

## 完了条件

* 直線辺 hover が回帰していない
* 曲線辺 hover が回帰していない
* 頂点上にマウスがあるとき，辺ではなく頂点が優先される
* Delete / Colorize / Normal モードで挙動が変わらない

## さらに改善する場合

必要になった段階で，以下を検討する．

* マウス位置が前フレームから変わっていない場合は再計算しない
* `hovered_edge_index` をキャッシュする
* 近傍候補だけを見るための空間インデックスを導入する

最初の段階では bounding box だけでよい．
実装が単純で，バグが入りにくい．
