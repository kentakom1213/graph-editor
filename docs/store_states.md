# ブラウザに状態をキャッシュ保存する方法

## 目的
0-indexed / 1-indexed、Undirected / Directed などの UI 状態をブラウザに保存し、再訪時に復元できるようにする。

## 対象となる状態
- 数値系: `indexBase` (0 or 1)
- 真偽値系: `isDirected` (true/false)
- 追加候補: 表示設定や直近の入力履歴など、軽量な UI 設定

## 方式候補

### localStorage
- 概要: 文字列の key-value を永続保存
- 期待容量: 5-10MB 程度 (ブラウザ依存)
- 長所: 実装が簡単、同期 API、少量の設定保存に向く
- 短所: 同期 API のため大量データには不向き、文字列のみ

### sessionStorage
- 概要: タブ単位で保持 (ブラウザを閉じると消える)
- 長所: セッション単位での状態保持に向く
- 短所: 再訪時の復元には向かない

### IndexedDB
- 概要: 非同期の構造化データ保存
- 長所: 容量が大きい、検索や構造化データに強い
- 短所: 実装コストが高い、今回の用途には過剰

### Cache Storage
- 概要: Service Worker で HTTP レスポンスをキャッシュ
- 長所: リソースキャッシュに有効
- 短所: UI 状態の保存には不適切

### URL (query / hash)
- 概要: 状態を URL に埋め込む
- 長所: 共有やブックマークに強い
- 短所: 文字数制限、常時の UI 設定保存には不向き

## 推奨方針
小さな UI 状態の保存は localStorage を基本とし、将来状態が増える場合は IndexedDB を検討する。

### 具体案
- 保存キー: `graph-editor:ui-state`
- 形式: JSON 文字列
- 例:

```json
{
  "version": 1,
  "indexBase": 0,
  "isDirected": false
}
```


## localStorage 実装方針 (Rust/WASM)

### 方針
- 本プロジェクトは eframe/egui を利用しているため、persistence 機能を使い `eframe::Storage` 経由で保存する
- 追加依存を避けたい場合は `web-sys` で localStorage を直接操作する

### 推奨ライブラリ/手段
- `eframe::Storage` + `eframe::get_value` / `eframe::set_value` (wasm ではブラウザの Storage に保存される)
- `web-sys` (すでに依存に含まれているため最小構成で利用可能)
- `gloo-storage` (型付きの薄いラッパーが欲しい場合の選択肢)
- `serde` / `serde_json` (構造体のシリアライズ/デシリアライズ)

### 例 (eframe Storage)

```rust
use serde::{Deserialize, Serialize};

const UI_STATE_KEY: &str = "graph-editor:ui-state";

#[derive(Debug, Clone, Serialize, Deserialize)]
struct UiState {
    version: u32,
    index_base: u8,
    is_directed: bool,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            version: 1,
            index_base: 0,
            is_directed: false,
        }
    }
}

impl UiState {
    fn load(storage: Option<&dyn eframe::Storage>) -> Self {
        eframe::get_value(storage, UI_STATE_KEY).unwrap_or_default()
    }

    fn store(&self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, UI_STATE_KEY, self);
    }
}
```

### 実装ポイント
- `CreationContext.storage` から読み込み、`App::save` または UI 変更時に保存する
- 変更頻度が高い場合は `dirty` フラグを立てて短い debounce を挟む
- 破損/不整合時は `Default` にフォールバックする

### 保存・復元のタイミング
- 保存: UI 変更時に保存 (必要に応じて debounce)
- 復元: アプリ起動時に読み込み、失敗時はデフォルト

### バージョニングと移行
- `version` フィールドを持たせ、形式変更時にマイグレーション
- 未知のフィールドは無視して安全に読み込む

### エラーハンドリング
- JSON parse の try/catch
- 容量超過時の例外を捕捉し、ユーザー操作は継続

### マルチタブ対応
- `storage` イベントで他タブの更新を反映 (必要なら)

## 使い分けの目安
- 軽量な UI 状態: localStorage
- セッション限定: sessionStorage
- 大きな履歴や構造化データ: IndexedDB
- 共有可能な状態: URL
