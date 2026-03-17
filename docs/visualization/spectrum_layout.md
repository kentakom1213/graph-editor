# スペクトルレイアウトによるグラフ描画アルゴリズム

無向グラフ $G=(V,E)$ が **辺リスト**の形で与えられたときに，
スペクトルレイアウトで 2 次元座標を計算する方法をまとめる．

スペクトルレイアウトでは，グラフラプラシアンの固有ベクトルを
頂点座標として用いる．

***

# 入力

* 頂点数 $n$
* 辺リスト\
  $$
  E = \{ (u_1,v_1), (u_2,v_2), \dots, (u_m,v_m) \}
  $$

ここで

$$
V = {0,1,\dots,n-1}
$$

とする．

***

# 出力

各頂点 $i \in V$ の 2 次元座標

$$
(x_i, y_i)
$$

***

# アルゴリズム

## Step 1: 隣接行列を作る

$$
A_{ij} =
\begin{cases}
1 & (i,j) \in E \\
0 & \text{otherwise}
\end{cases}
$$

***

## Step 2: 次数行列を作る

$$
D_{ii} = \deg(i)
$$

それ以外の成分は 0．

***

## Step 3: ラプラシアン行列を作る

$$
L = D - A
$$

これは $n \times n$ の対称行列になる．

***

## Step 4: 固有値分解

ラプラシアンの固有値分解を行う：

$$
L v_k = \lambda_k v_k
$$

固有値を昇順に並べる：

$$
0 = \lambda_1 \le \lambda_2 \le \cdots \le \lambda_n
$$

***

## Step 5: 座標を決定

第 2，第 3 固有ベクトルを用いる：

$$
x_i = v_2(i)
$$

$$
y_i = v_3(i)
$$

これにより頂点 $i$ の座標が得られる．

***

## Step 6: スケーリング（任意）

可視化しやすいように

* 平行移動
* スケーリング

を行う．

例えば

$$
x_i \leftarrow \frac{x_i - \bar{x}}{\max_j |x_j|}
$$

などで正規化する．

***

# 疑似コード

```text
SpectralLayout(n, E):

    A ← n×n zero matrix

    for (u,v) in E:
        A[u][v] ← 1
        A[v][u] ← 1

    D ← diagonal matrix
    for i in 0..n-1:
        D[i][i] ← degree(i)

    L ← D − A

    (λ, V) ← eigen_decomposition(L)

    sort eigenvalues

    v2 ← eigenvector corresponding to λ2
    v3 ← eigenvector corresponding to λ3

    for i in 0..n-1:
        x[i] ← v2[i]
        y[i] ← v3[i]

    return {(x[i], y[i])}
```

***

# 計算量

* ラプラシアン構築
  $$
  O(n+m)
  $$

* 固有値分解（密行列）
  $$
  O(n^3)
  $$

実用実装では

* Lanczos 法
* power iteration

を使うことで

$$
O(km)
$$

程度で計算できる．

***

# 特徴

長所

* 実装が簡単
* グラフの大域構造が現れる
* 初期配置として優秀

短所

* 辺交差を最小化しない
* 密グラフでは構造が潰れやすい

***

# 実装方針

本グラフ描画ツールでは

$$
\text{spectral layout}
\rightarrow
\text{force-directed refinement}
$$

という形で利用する．

スペクトルレイアウトで初期配置を作り，
力学モデルで見た目を調整する．
