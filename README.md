# Graph Editor

[![Build Status](https://github.com/kentakom1213/graph-editor/workflows/CI/badge.svg)](https://github.com/kentakom1213/graph-editor/actions?workflow=CI)

Graph Editor は [eframe](https://github.com/emilk/egui/tree/master/crates/eframe) と [egui](https://github.com/emilk/egui/) によるグラフ編集アプリです．

![demo](./images/graph-editor-demo.gif)

## 📌 操作

### Edit Mode

| モード                       | コマンド | 説明                              |
| :--------------------------- | :------: | :-------------------------------- |
| Normal モード                |    N     | 頂点の移動などを行う              |
| Add Vertex (頂点追加) モード |    V     | クリックした位置に頂点を追加する  |
| Add Edge (辺追加) モード     |    E     | 選択した 2 つの頂点の間に辺を張る |
| Delete Edge (辺削除) モード  |    D     | クリックした頂点/辺を削除する     |

### Indexing

頂点の表示方法を変更する．

| Indexing  |  コマンド  | 説明                        |
| :-------- | :--------: | :-------------------------- |
| 0-indexed | 1 (toggle) | 頂点を `0` 始まりで表示する |
| 1-indexed | 1 (toggle) | 頂点を `1` 始まりで表示する |

### Direction

| Direction  |      コマンド      | 説明                     |
| :--------- | :----------------: | :----------------------- |
| Undirected | Shift + D (toggle) | 無向グラフとして描画する |
| Directed   | Shift + D (toggle) | 有向グラフとして描画する |

---

## 🚀 ローカル環境での実行方法

```bash
# リポジトリをクローン
git clone https://github.com/powell/graph-editor.git
cd graph-editor

# アプリケーションをビルドして実行
cargo run --release
```

---

## 🌐 Web 版をローカルでプレビューする方法

### ✅ インストール

- Rust と Trunk のインストールが必要です。

```bash
# WebAssemblyターゲットを追加
rustup target add wasm32-unknown-unknown

# Trunk をインストール
cargo install --locked trunk
```

### 🚧 ローカルサーバーで実行する場合

```bash
# ローカルサーバーを起動
trunk serve
```

ブラウザで `http://127.0.0.1:8080` を開いて確認してください。

## 🤝 コントリビューション

バグ報告、機能追加の提案、プルリクエストなど歓迎いたします。

## 📄 ライセンス

このプロジェクトは MIT ライセンス、APACHE ライセンスの下で提供されています。詳細は [LICENSE-APACHE](https://github.com/kentakom1213/graph-editor/blob/main/LICENSE-APACHE)、[LICENSE-MIT](https://github.com/kentakom1213/graph-editor/blob/main/LICENSE-MIT) ファイルを参照してください。
