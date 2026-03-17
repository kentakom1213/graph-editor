# アフィン変換（移動・拡大縮小）の実装概要

このプロジェクトでは，グラフ全体の移動・拡大縮小を `Affine2D` で管理する．
各頂点の座標はローカル座標として保持し，描画・入力時にアフィン変換を適用する．

## 基本構造

- `src/math/affine.rs`
  - `Affine2D`: 3x3 行列で表現する 2D アフィン変換
  - `from_transition`: 平行移動
  - `from_center_and_scale`: 任意中心での拡大縮小
  - `try_compose`: 合成時にスケール範囲をチェック
  - `inverse`: 逆変換（入力座標の変換に使用）
  - `ApplyAffine`: `egui::Vec2` と `egui::Pos2` に適用

## 移動（ドラッグ）

- `src/components/transition_and_scale.rs`
  - 左ドラッグの差分 `delta` を現在スケールで割り，`Affine2D::from_transition` で合成する．
  - スケール済み座標系での移動量を整合させるため，`delta / cur_scale` を使う．

## 拡大縮小（ホイール・ピンチ）

- `src/components/transition_and_scale.rs`
  - 現在のアフィンを取得して逆変換を求める．
  - マウス位置（またはピンチ中心）をローカル座標に戻し，その点を中心にスケール行列を作る．
  - `try_compose` で `scale_min`〜`scale_max` を超えない場合のみ更新する．

## 入力座標の補正

- `src/components/central_panel.rs`
  - クリック位置を逆変換でローカルに戻した上で，アフィンの平行移動成分を加算して座標を確定する．
- `src/graph/structures.rs`
  - 追加する頂点の座標から現在の平行移動成分を引いてローカル座標として保存する．

## 描画時の適用

- `src/graph/structures.rs`
  - `Vertex::get_position` がアフィンを適用した描画用座標を返す．
  - `Vertex::update_position` は逆変換でローカルに保存する．

## 回転の実装方針

回転を追加する場合は `Affine2D` に回転行列の生成と合成を組み込み，入力座標の補正と UI 操作を追加する．

- `src/math/affine.rs`
  - 回転行列 `from_center_and_rotation(center, rad)` を追加する．
  - 既存の `try_compose` で合成できるように，スケール制約はそのまま維持する．
  - 逆変換 `inverse` は回転を含む 2x2 の線形部分を反転できるため，そのまま利用できる．
- `src/components/transition_and_scale.rs`
  - 回転操作（例: `Shift + ホイール` や `R` キーでドラッグ）の入力を追加する．
  - 現在のアフィンの逆変換で中心点をローカルに戻し，`from_center_and_rotation` を合成する．
- `src/components/central_panel.rs`
  - クリック位置の逆変換が回転を含むようになるため，`inverse` での補正を前提に維持する．

回転の中心は現在のマウス位置（または画面中心）を基準にする設計が扱いやすい．
