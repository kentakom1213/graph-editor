# 実装依頼: Graph Editor 独自の JSON 保存形式を追加する

## 目的

Graph Editor に，グラフ構造を JSON として保存・読み込みできる独自フォーマットを追加してください．

この保存形式では，以下を扱えるようにしてください．

- フォーマット名
- バージョン情報
- 有向グラフ / 無向グラフの情報
- 0-indexed / 1-indexed の情報
- 頂点一覧
- 辺一覧
- 頂点と辺の接続関係
- 頂点の位置情報
- 頂点の色情報
- 辺の色情報
- ただし，頂点の位置情報・色情報・辺の色情報は，保存時に含める / 含めないを切り替えられるようにする

## 方針

保存形式は JSON を主形式としてください．

YAML / TOML は今回実装しなくてよいです．
ただし，将来的に拡張できるよう，保存形式の実装は `export` または `io` 系のモジュールに分離してください．

既存のコード構造を確認したうえで，できるだけ既存の `Graph` / `Vertex` / `Edge` / `GraphEditorApp` に自然に接続してください．

## 保存形式

ファイル全体は次の形にしてください．

```json
{
  "format": "graph-editor",
  "version": 1,
  "graph": {
    "directed": false,
    "index_origin": 0,
    "features": {
      "vertex_position": true,
      "vertex_style": true,
      "edge_style": true
    },
    "vertices": [
      {
        "id": 0,
        "label": "0",
        "position": {
          "x": 120.0,
          "y": 80.0
        },
        "style": {
          "fill": "#ffffff",
          "stroke": "#000000",
          "text": "#000000"
        }
      }
    ],
    "edges": [
      {
        "id": 0,
        "from": 0,
        "to": 1,
        "label": "",
        "style": {
          "stroke": "#000000",
          "text": "#000000"
        }
      }
    ]
  }
}
```

## 構造だけ保存する場合

位置情報・色情報を含めない場合は，次のような JSON を出力してください．

```json
{
  "format": "graph-editor",
  "version": 1,
  "graph": {
    "directed": false,
    "index_origin": 0,
    "features": {
      "vertex_position": false,
      "vertex_style": false,
      "edge_style": false
    },
    "vertices": [
      {
        "id": 0,
        "label": "0"
      },
      {
        "id": 1,
        "label": "1"
      }
    ],
    "edges": [
      {
        "id": 0,
        "from": 0,
        "to": 1,
        "label": ""
      }
    ]
  }
}
```

## 重要な仕様

### `format`

`format` は必ず `"graph-editor"` にしてください．

読み込み時に `format` が `"graph-editor"` でない場合は，エラーにしてください．

### `version`

`version` は整数で，初期値は `1` にしてください．

読み込み時は，まず `version == 1` のみ対応してください．
未対応バージョンの場合は，panic ではなくエラーを返してください．

### `directed`

`graph.directed` は，有向グラフかどうかを表します．

- `true`: 有向グラフ
- `false`: 無向グラフ

無向グラフの場合でも，辺は `from` / `to` を持ちます．
この場合，`from` / `to` は保存上の便宜的な向きとして扱ってください．

### `index_origin`

`graph.index_origin` は，ユーザーが想定する頂点番号の開始位置を表します．

- `0`: 0-indexed
- `1`: 1-indexed

ただし，保存形式内の `id`, `from`, `to` は常に内部 ID として扱ってください．
`index_origin` は，表示や競技プログラミング向け出力時の設定として扱います．

つまり，読み込み時に `index_origin == 1` だからといって，`id`, `from`, `to` を勝手に 1 減らさないでください．

### `features`

`graph.features` は，このファイルにどの付加情報が含まれているかを表します．

```json
{
  "vertex_position": true,
  "vertex_style": true,
  "edge_style": true
}
```

意味は次の通りです．

- `vertex_position`: 頂点の位置情報を含むか
- `vertex_style`: 頂点の色情報を含むか
- `edge_style`: 辺の色情報を含むか

ただし，読み込み時は `features` だけを信用しないでください．
各頂点・各辺の `position` / `style` が実際に存在するかも確認してください．

例えば，`vertex_position == true` でも一部の頂点に `position` がない場合は，その頂点だけデフォルト配置にしてください．

### `vertices`

`graph.vertices` は頂点の配列です．

各頂点は最低限，次の情報を持ちます．

```json
{
  "id": 0
}
```

追加で，以下を持てます．

```json
{
  "label": "0",
  "position": {
    "x": 120.0,
    "y": 80.0
  },
  "style": {
    "fill": "#ffffff",
    "stroke": "#000000",
    "text": "#000000"
  }
}
```

`position` は optional にしてください．
`style` も optional にしてください．

保存時に位置情報を含めない設定の場合，`position` フィールドは出力しないでください．

保存時に頂点色情報を含めない設定の場合，頂点の `style` フィールドは出力しないでください．

### `edges`

`graph.edges` は辺の配列です．

各辺は最低限，次の情報を持ちます．

```json
{
  "id": 0,
  "from": 0,
  "to": 1
}
```

追加で，以下を持てます．

```json
{
  "label": "",
  "style": {
    "stroke": "#000000",
    "text": "#000000"
  }
}
```

`style` は optional にしてください．

保存時に辺色情報を含めない設定の場合，辺の `style` フィールドは出力しないでください．

## Rust の型設計

以下のような保存用 DTO を追加してください．
既存の内部構造体に直接 `Serialize` / `Deserialize` を生やすより，保存形式専用の型に変換する方針にしてください．

```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GraphFile {
    pub format: String,
    pub version: u32,
    pub graph: GraphData,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GraphData {
    pub directed: bool,
    pub index_origin: u8,
    pub features: GraphFeatures,
    pub vertices: Vec<VertexData>,
    pub edges: Vec<EdgeData>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GraphFeatures {
    pub vertex_position: bool,
    pub vertex_style: bool,
    pub edge_style: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VertexData {
    pub id: usize,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<PositionData>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<VertexStyleData>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EdgeData {
    pub id: usize,
    pub from: usize,
    pub to: usize,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<EdgeStyleData>,
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct PositionData {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VertexStyleData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fill: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub stroke: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EdgeStyleData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stroke: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
}
```

## 保存オプション

保存時に，次のようなオプションを受け取れるようにしてください．

```rust
#[derive(Debug, Clone, Copy)]
pub struct SaveOptions {
    pub include_vertex_position: bool,
    pub include_vertex_style: bool,
    pub include_edge_style: bool,
}
```

デフォルトでは，すべて `true` にしてください．

```rust
impl Default for SaveOptions {
    fn default() -> Self {
        Self {
            include_vertex_position: true,
            include_vertex_style: true,
            include_edge_style: true,
        }
    }
}
```

## 必要な関数

次のような関数を追加してください．
実際の引数型は既存コードに合わせて調整して構いません．

```rust
pub fn export_graph_to_file(graph: &Graph, options: SaveOptions) -> GraphFile;
pub fn export_graph_to_json(graph: &Graph, options: SaveOptions) -> Result<String, ExportError>;
pub fn import_graph_from_file(file: GraphFile) -> Result<Graph, ImportError>;
pub fn import_graph_from_json(json: &str) -> Result<Graph, ImportError>;
```

`serde_json::to_string_pretty` を使って，人間が読みやすい JSON を出力してください．

## 保存時の仕様

保存時は，次のようにしてください．

- `format` は `"graph-editor"`
- `version` は `1`
- `graph.directed` は現在のグラフ設定を反映する
- `graph.index_origin` は現在の表示・出力設定を反映する
- `features.vertex_position` は `SaveOptions::include_vertex_position`
- `features.vertex_style` は `SaveOptions::include_vertex_style`
- `features.edge_style` は `SaveOptions::include_edge_style`
- `include_vertex_position == false` の場合，すべての頂点で `position` を `None` にする
- `include_vertex_style == false` の場合，すべての頂点で `style` を `None` にする
- `include_edge_style == false` の場合，すべての辺で `style` を `None` にする

## 読み込み時の仕様

読み込み時は，次を検証してください．

- `format == "graph-editor"` であること
- `version == 1` であること
- `index_origin` が `0` または `1` であること
- 頂点 `id` に重複がないこと
- 各辺の `from` / `to` が存在する頂点 `id` を参照していること
- 辺 `id` に重複がないこと

検証に失敗した場合は，panic せずに `Result::Err` を返してください．

`position` がない頂点は，読み込み後にデフォルト配置を与えてください．
デフォルト配置は，既存の頂点追加時の配置処理があればそれに合わせてください．
なければ，円形配置やグリッド配置などの簡単な配置で構いません．

`style` がない頂点・辺は，現在のデフォルト色を使ってください．

## エラー型

少なくとも次のようなエラーを区別してください．
`thiserror` を既に使っていなければ，単純な enum と `Display` 実装で構いません．

```rust
#[derive(Debug)]
pub enum ImportError {
    InvalidJson(String),
    InvalidFormat(String),
    UnsupportedVersion(u32),
    InvalidIndexOrigin(u8),
    DuplicateVertexId(usize),
    DuplicateEdgeId(usize),
    MissingVertex { edge_id: usize, vertex_id: usize },
}
```

```rust
#[derive(Debug)]
pub enum ExportError {
    SerializeFailed(String),
}
```

## UI 要件

エクスポート UI に，次のチェックボックスを追加してください．

```text
[x] 頂点の位置を保存する
[x] 頂点の色を保存する
[x] 辺の色を保存する
```

チェックボックスの状態に応じて `SaveOptions` を作り，JSON エクスポートに渡してください．

可能であれば，インポート UI も追加してください．
ただし，既存のファイル入出力の仕組みが未整備なら，まずは JSON 文字列をコピー・ペーストで読み書きできる形でも構いません．

## 互換性の注意

既存の競技プログラミング向け出力形式とは分離してください．

この JSON 保存形式は，Graph Editor のプロジェクト保存用です．
競技プログラミング向けの辺リスト出力とは目的が違います．

そのため，`index_origin` は保存ファイル内の `id` を変換するためではなく，表示・出力設定を復元するための情報として扱ってください．

## テスト

以下のテストを追加してください．

### 1. 構造だけ保存できる

`SaveOptions` ですべて `false` にしたとき，出力 JSON に `position` と `style` が含まれないことを確認してください．

### 2. 位置と色を含めて保存できる

`SaveOptions` ですべて `true` にしたとき，出力 JSON に `position` と `style` が含まれることを確認してください．

### 3. 読み込み後に辺の接続関係が保たれる

JSON を読み込んだあと，辺の `from` / `to` が元の頂点を参照していることを確認してください．

### 4. 存在しない頂点を参照する辺はエラーになる

例えば，頂点が `0` しかないのに，辺が `to: 1` を参照している JSON は `ImportError::MissingVertex` にしてください．

### 5. 重複する頂点 ID はエラーになる

同じ `id` を持つ頂点が複数ある場合は，`ImportError::DuplicateVertexId` にしてください．

### 6. 未対応バージョンはエラーになる

`version: 999` の JSON は `ImportError::UnsupportedVersion(999)` にしてください．

### 7. `index_origin` の値を検証する

`index_origin` が `0` または `1` 以外の場合は `ImportError::InvalidIndexOrigin` にしてください．

## 実装上の注意

- 既存の内部データ構造を壊さないでください
- 保存形式専用 DTO と内部データ構造の相互変換を実装してください
- `serde_json` を使ってください
- `position` / `style` は `Option` と `skip_serializing_if` を使って，保存しない設定のときフィールド自体を出さないでください
- 読み込み時は，古い JSON や手書き JSON でも panic しないようにしてください
- 不正な JSON ではユーザーに分かりやすいエラーメッセージを返してください

## 完了条件

以下が満たされれば完了です．

- JSON 形式でグラフをエクスポートできる
- JSON 形式からグラフをインポートできる
- 有向 / 無向の情報を保存・復元できる
- 0-indexed / 1-indexed の情報を保存・復元できる
- 頂点・辺の接続関係を保存・復元できる
- 頂点の位置情報を保存する / しないを切り替えられる
- 頂点の色情報を保存する / しないを切り替えられる
- 辺の色情報を保存する / しないを切り替えられる
- 無効な JSON に対して panic せずエラーを返せる
- 上記テストが通る
