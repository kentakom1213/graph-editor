# グラフ描画アルゴリズム

## 資料

- [力学モデル (グラフ描画アルゴリズム) - wikipedia](https://ja.wikipedia.org/wiki/%E5%8A%9B%E5%AD%A6%E3%83%A2%E3%83%87%E3%83%AB_(%E3%82%B0%E3%83%A9%E3%83%95%E6%8F%8F%E7%94%BB%E3%82%A2%E3%83%AB%E3%82%B4%E3%83%AA%E3%82%BA%E3%83%A0))

## 定義

- グラフ $G = (V, E)$ 
  - 頂点集合 $V$ （ $N := |V|$ ）
    - 頂点 $v\in V$ について，
      - $v.\!\boldsymbol{x}$ ：頂点 $v$ の座標
      - $v.\!\boldsymbol{\dot{x}}$ ：頂点 $v$​ の速度ベクトル
      - $v.\!m$ ：頂点 $v$ の重さ
  - 辺集合 $E \subseteq \binom{V}{2}$ （ $M := |E|$ ）
- 定数
  - $c$ ：クーロン定数
  - $k$​ ：ばね定数
  - $l$​ ：ばねの自然長
  - $h$ ：減衰定数（ $0 < h < 1$​ ）
  - $\Delta t$ ：微小時間（フレームレートを取得）


## アルゴリズム

以下のようなアルゴリズムで描画を行う．

1. 全ての頂点 $v\in V$ について， $v.\!\mathit{dx} := 0,~v.\!\mathit{dy} := 0$ とする．

2. 全ての頂点 $v\in V$ について，

   1. 頂点 $v$ に加わる力を $\boldsymbol{f}_v := (0, 0)$​ とする．

   1. 全ての頂点 $w\in V$ について，$v.\!\boldsymbol{x}$ から $w.\!\boldsymbol{x}$ へ向かう単位ベクトルを $\hat{r}$ とするとき，
      $$
      \boldsymbol{f}_v := \boldsymbol{f}_v + \frac{c}{\|v.\!\boldsymbol{x} - w.\!\boldsymbol{x}\|^2}\cdot \hat{r}.
      $$

   2. 頂点 $v$ の隣接頂点 $w\in N(v)$ について，
      $$
      \boldsymbol{f}_v := \boldsymbol{f}_v + k \cdot (\|v.\!\boldsymbol{x} - w.\!\boldsymbol{x}\| - l) \cdot \hat{r}.
      $$

   3. 振動の減衰を加味して速度を更新する．
      $$
      v.\!\boldsymbol{\dot{x}} := h \cdot \left(v.\!\boldsymbol{\dot{x}} + \Delta t \cdot \frac{\boldsymbol{f}_v}{v.\!m} \right).
      $$

   4. 頂点の位置を更新する．
      $$
      v.\!\boldsymbol{x} := v.\!\boldsymbol{x} + \Delta t \cdot v.\!\boldsymbol{\dot{x}}.
      $$
