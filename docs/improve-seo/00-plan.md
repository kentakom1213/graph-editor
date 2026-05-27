# SEO 改善メモ for graph-editor

## 背景

現在，`https://kentakom1213.github.io/graph-editor/` は `site:` 検索では見つかるため，Google にまったく認識されていないわけではない．

一方で，通常の検索語，たとえば `Graph Editor` や `グラフエディタ` では上位に出てこない．主な原因は次の 2 点と考えられる．

- `Graph Editor` が一般名詞に近く，競合が非常に多い
- WASM / SPA アプリのため，クローラから見える初期 HTML の本文が薄い

そのため，検索エンジンに対して「これは競技プログラミング向けのグラフ作成ツールである」と静的に伝わるようにする．

## 目的

この修正の目的は，以下の検索語で見つかりやすくすることである．

- `competitive programming graph editor`
- `graph editor for competitive programming`
- `AtCoder graph editor`
- `競プロ グラフエディタ`
- `競技プログラミング グラフ 作成`
- `AtCoder グラフ 作成`

`Graph Editor` 単体での上位表示は競合が強いため，まずは用途を含むロングテール検索での流入を狙う．

## 修正方針

### 1. HTML の title を用途込みにする

現在の `<title>` が単に `Graph Editor` のようになっている場合，検索語として一般的すぎる．

次のように，競技プログラミング向けツールであることを含める．

```html
<title>Graph Editor - Competitive Programming Graph Tool</title>
```

日本語寄りにする場合は次でもよい．

```html
<title>Graph Editor - 競技プログラミング向けグラフ作成ツール</title>
```

英語検索も拾いたいので，基本は英語を含めた前者が無難．

### 2. meta description を追加する

`index.html` の `<head>` に `meta description` を追加する．

```html
<meta
  name="description"
  content="Graph Editor is a graph creation and visualization tool for competitive programming. Create, edit, and export graphs for AtCoder and programming contests."
/>
```

日本語も入れるなら，少し長くなるが次のようにする．

```html
<meta
  name="description"
  content="Graph Editor is a graph creation and visualization tool for competitive programming. 競技プログラミング向けにグラフを作成・編集・可視化できるツールです。"
/>
```

### 3. 初期 HTML に静的な説明文を入れる

WASM / SPA アプリでは，JavaScript 実行後に UI が描画されるため，検索エンジンから見える初期 HTML の本文が薄くなりやすい．

`index.html` の `<body>` 内に，静的な説明文を追加する．

例：

```html
<main id="app">
  <section>
    <h1>Graph Editor</h1>
    <p>
      Graph Editor is a graph creation and visualization tool for competitive
      programming. It helps users create directed and undirected graphs, edit
      vertices and edges, and export graphs in programming-contest input
      formats.
    </p>
    <p>
      競技プログラミング向けのグラフ作成・可視化ツールです。
      頂点や辺を直感的に編集し、AtCoder などで使いやすい入力形式に変換できます。
    </p>
  </section>
</main>
```

既存の WASM 起動処理が `#app` などを使っている場合は，既存の構造を壊さないように注意する．

もし WASM 側が `body` 全体や特定の DOM を置き換える場合でも，初期 HTML に一度は説明文が含まれていればよい．

### 4. noscript に説明文を入れる

JavaScript が実行されない環境向けに，`<noscript>` も追加する．

```html
<noscript>
  <section>
    <h1>Graph Editor</h1>
    <p>
      Graph Editor is a graph creation and visualization tool for competitive
      programming. Please enable JavaScript to use the interactive editor.
    </p>
    <p>
      競技プログラミング向けのグラフ作成・可視化ツールです。
      インタラクティブなエディタを利用するには JavaScript を有効にしてください。
    </p>
  </section>
</noscript>
```

### 5. OGP / Twitter Card を追加する

SNS やブログから共有されたときに内容が伝わりやすいように，OGP を追加する．

```html
<meta
  property="og:title"
  content="Graph Editor - Competitive Programming Graph Tool"
/>
<meta
  property="og:description"
  content="Create, edit, and visualize graphs for competitive programming and AtCoder."
/>
<meta property="og:type" content="website" />
<meta
  property="og:url"
  content="https://kentakom1213.github.io/graph-editor/"
/>
```

Twitter Card も追加する．

```html
<meta name="twitter:card" content="summary" />
<meta
  name="twitter:title"
  content="Graph Editor - Competitive Programming Graph Tool"
/>
<meta
  name="twitter:description"
  content="Create, edit, and visualize graphs for competitive programming and AtCoder."
/>
```

画像がある場合は，次も追加する．

```html
<meta
  property="og:image"
  content="https://kentakom1213.github.io/graph-editor/ogp.png"
/>
<meta
  name="twitter:image"
  content="https://kentakom1213.github.io/graph-editor/ogp.png"
/>
```

### 6. canonical URL を追加する

URL の正規化のため，`canonical` を追加する．

```html
<link rel="canonical" href="https://kentakom1213.github.io/graph-editor/" />
```

末尾スラッシュありに統一する．

### 7. README から GitHub Pages へのリンクを目立たせる

GitHub リポジトリ側は検索に出やすいため，README の上部にデモリンクを置く．

例：

```markdown
# Graph Editor

Graph Editor is a graph creation and visualization tool for competitive programming.

Demo: https://kentakom1213.github.io/graph-editor/

## Features

- Create and edit vertices and edges interactively
- Visualize directed and undirected graphs
- Export graphs for programming contest inputs
- Useful for AtCoder and competitive programming
```

README にも次の語を自然に含める．

- `competitive programming`
- `AtCoder`
- `graph editor`
- `graph visualization`
- `graph creation tool`

### 8. サイト内に GitHub リポジトリへのリンクを置く

アプリ画面内，または初期 HTML の説明文中に GitHub リポジトリへのリンクを置く．

```html
<p>
  Source code is available on
  <a href="https://github.com/kentakom1213/graph-editor">GitHub</a>.
</p>
```

これにより，GitHub Pages と GitHub リポジトリの関連が検索エンジンに伝わりやすくなる．

### 9. sitemap.xml を追加する

`public/sitemap.xml` など，ビルド後にルートへ配置される場所に sitemap を追加する．

```xml
<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
  <url>
    <loc>https://kentakom1213.github.io/graph-editor/</loc>
  </url>
</urlset>
```

### 10. robots.txt を追加する

`robots.txt` を追加し，クロールを許可する．

```txt
User-agent: *
Allow: /

Sitemap: https://kentakom1213.github.io/graph-editor/sitemap.xml
```

## 期待する変更ファイル

想定される変更ファイルは次の通り．

```txt
index.html
public/sitemap.xml
public/robots.txt
README.md
```

プロジェクト構成によっては，`index.html` や `public/` の位置は適宜読み替える．

## 実装時の注意

既存の WASM / eframe / trunk の起動処理を壊さないこと．

特に，次の点に注意する．

- WASM が mount する要素の `id` を変更しない
- 既存の `script` タグを削除しない
- `base` タグがある場合，GitHub Pages のサブパス `/graph-editor/` で壊れないようにする
- `og:url`，`canonical`，`sitemap.xml` の URL は末尾スラッシュありで統一する

## 完了条件

次の状態になっていれば完了とする．

- `index.html` に適切な `<title>` がある
- `index.html` に `meta description` がある
- 初期 HTML に，競技プログラミング向けグラフエディタであることを説明する静的本文がある
- OGP / Twitter Card が設定されている
- `canonical` が設定されている
- `sitemap.xml` が追加されている
- `robots.txt` が追加されている
- README の上部から公開ページへリンクされている

## 修正後に行うこと

修正をデプロイした後，Google Search Console で以下を行う．

1. `https://kentakom1213.github.io/graph-editor/` を URL 検査する
2. インデックス登録をリクエストする
3. `sitemap.xml` を送信する
4. 数日後に `site:kentakom1213.github.io/graph-editor` で反映を確認する

検索順位の改善には時間がかかるため，デプロイ直後に通常検索で上位表示されるとは限らない．
まずは `site:` 検索で title / description が期待通りに見えるかを確認する．
