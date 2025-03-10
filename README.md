# Graph Editor

[![Build Status](https://github.com/kentakom1213/graph-editor/workflows/CI/badge.svg)](https://github.com/kentakom1213/graph-editor/actions?workflow=CI)

Graph Editor は [eframe](https://github.com/emilk/egui/tree/master/crates/eframe) と [egui](https://github.com/emilk/egui/) によるグラフ編集アプリです．

## 📌 主な機能

- **頂点の追加と移動**: クリックで新しい頂点を追加し、ドラッグで移動可能。
- **辺の作成**: 頂点同士を接続して辺を追加。
- **要素の選択と削除**: 頂点や辺を選択して削除可能。
- **編集モードの切り替え**:
  - Normal モード
  - Add Vertex (頂点追加) モード
  - Add Edge (辺追加) モード
  - Delete Edge (辺削除) モード
- **ショートカットキー対応**:
  - `V`: 頂点追加モード
  - `E`: 辺追加モード
  - `D`: 辺削除モード
  - `Esc`: 通常モードに戻る
- **ステータスパネル**: 編集モードや情報をコード形式で表示、コピー可能。

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

## 🌐 Web版をローカルでプレビューする方法

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


