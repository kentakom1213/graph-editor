# PR-37 追加 SEO 修正指示

## 背景

PR-37 により，`https://kentakom1213.github.io/graph-editor/pr-37/` の初期 HTML に，検索エンジンから読める静的な説明文が追加された．

現在，初期 HTML から次の内容が確認できる．

- `Graph Editor` の見出し
- 英語の説明文
- 日本語の説明文
- GitHub リポジトリへのリンク
- JavaScript 無効時向けの説明
- `Loading…`

これは，修正前に比べてかなり良い状態である．一方で，通常説明と `noscript` 用説明がどちらもテキストとして見えており，`Graph Editor` の見出しと説明文がやや重複している．また，PR プレビュー URL と本番 URL の扱いを明確にしておく必要がある．

## 目的

この追加修正では，以下を目的とする．

- 静的説明文の重複を減らす
- 本番 URL を canonical として明示する
- PR プレビュー URL が検索結果に出ないようにする
- `sitemap.xml` と `robots.txt` を本番 URL 基準で整える
- 既存の WASM / trunk / eframe の起動処理を壊さない

## 追加修正 1: 通常説明と noscript 説明の重複を整理する

現在，初期 HTML 上で `Graph Editor` 見出しが 2 回出ている．

通常の静的説明と `noscript` 説明がほぼ同じ内容になっているため，検索エンジンから見た本文がやや重複気味である．

通常表示用の説明は詳しめに残し，`noscript` 側は短くする．

例：

```html
<main id="app">
  <section class="seo-description">
    <h1>Graph Editor</h1>
    <p>
      Graph Editor is a graph creation and visualization tool for competitive
      programming. It helps users create directed and undirected graphs, edit
      vertices and edges, and export graphs in programming-contest input
      formats.
    </p>
    <p>
      競技プログラミング向けのグラフ作成・可視化ツールです．
      頂点や辺を直感的に編集し，AtCoder などで使いやすい入力形式に変換できます．
    </p>
    <p>
      Source code is available on
      <a href="https://github.com/kentakom1213/graph-editor">GitHub</a>.
    </p>
  </section>
</main>

<noscript>
  <p>
    Please enable JavaScript to use Graph Editor. JavaScript を有効にすると
    Graph Editor を利用できます．
  </p>
</noscript>
```

重要なのは，`noscript` 内に再度 `<h1>Graph Editor</h1>` を置かないこと．
`h1` は通常説明側の 1 個に寄せる．

## 追加修正 2: canonical URL を本番 URL に固定する

PR プレビュー URL は，

```txt
https://kentakom1213.github.io/graph-editor/pr-37/
```

である．

しかし，検索エンジンに正規ページとして扱ってほしいのは本番 URL である．

```txt
https://kentakom1213.github.io/graph-editor/
```

そのため，`index.html` の `<head>` に次を入れる．

```html
<link rel="canonical" href="https://kentakom1213.github.io/graph-editor/" />
```

PR プレビューでも本番でも，canonical は本番 URL を指すようにする．

## 追加修正 3: PR プレビューを noindex にする

PR プレビューは確認用であり，検索結果に出てほしくない．

可能であれば，PR プレビュー環境のみ次を追加する．

```html
<meta name="robots" content="noindex, nofollow" />
```

ただし，本番ページにはこのタグを入れてはいけない．

本番ページに `noindex` が入ると，検索結果から除外される可能性がある．

実装方針は，ビルド時の環境変数で切り替えるのがよい．

例：

```html
<!-- PR preview only -->
<meta name="robots" content="noindex, nofollow" />
```

もし環境ごとの HTML 差し替えが難しい場合は，少なくとも canonical を本番 URL に向ける．

優先度は次の通り．

1. 本番に `noindex` を絶対に入れない
2. PR プレビューには `noindex` を入れる
3. すべての環境で canonical は本番 URL に向ける

## 追加修正 4: OGP URL も本番 URL に固定する

`og:url` が PR プレビュー URL になっている場合，本番 URL に修正する．

```html
<meta
  property="og:url"
  content="https://kentakom1213.github.io/graph-editor/"
/>
```

合わせて，タイトルと説明は次のようにする．

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

Twitter Card も同様に設定する．

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

## 追加修正 5: sitemap.xml を本番 URL だけにする

`sitemap.xml` は本番 URL のみを載せる．
PR プレビュー URL は載せない．

```xml
<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
  <url>
    <loc>https://kentakom1213.github.io/graph-editor/</loc>
  </url>
</urlset>
```

配置先は，ビルド後に次でアクセスできる場所にする．

```txt
https://kentakom1213.github.io/graph-editor/sitemap.xml
```

## 追加修正 6: robots.txt を本番 URL 基準にする

`robots.txt` はクロールを許可し，sitemap を明示する．

```txt
User-agent: *
Allow: /

Sitemap: https://kentakom1213.github.io/graph-editor/sitemap.xml
```

配置先は，ビルド後に次でアクセスできる場所にする．

```txt
https://kentakom1213.github.io/graph-editor/robots.txt
```

ただし，GitHub Pages のサブパス配信では，`robots.txt` の扱いがルートパスと混ざる可能性があるため，実際にデプロイ後の URL でアクセスできるか確認すること．

## 追加修正 7: hidden にしすぎない

静的説明文を完全に `display: none` にすると，検索エンジンから不自然に見える可能性がある．

WASM 起動前に表示され，起動後にアプリ UI に置き換わる程度なら問題ない．

避けたい例：

```css
.seo-description {
  display: none;
}
```

許容しやすい例：

```css
.seo-description {
  padding: 1rem;
}
```

または，WASM の初期化後にアプリが自然に置き換える構造にする．

## 追加修正 8: README のデモリンク文言を検索語に寄せる

README の冒頭に，本番ページへのリンクを置く．

```markdown
# Graph Editor

Graph Editor is a graph creation and visualization tool for competitive programming.

Demo: https://kentakom1213.github.io/graph-editor/

Useful for creating graphs for AtCoder and programming contests.
```

`Demo` だけでなく，`competitive programming`，`AtCoder`，`graph creation` が自然に入るようにする．

## 期待する変更ファイル

想定される変更ファイルは次の通り．

```txt
index.html
public/sitemap.xml
public/robots.txt
README.md
```

プロジェクト構成によっては，`public/` のパスは実際の trunk / GitHub Pages の配置に合わせて読み替える．

## 完了条件

次を満たせば完了とする．

- 初期 HTML に静的な英語説明と日本語説明がある
- `<h1>Graph Editor</h1>` が重複していない
- `<meta name="description">` がある
- `<link rel="canonical">` が本番 URL を指している
- `og:url` が本番 URL を指している
- PR プレビューに `noindex` を入れられる場合は入っている
- 本番ページには `noindex` が入っていない
- `sitemap.xml` に PR プレビュー URL が含まれていない
- `robots.txt` が sitemap を指している
- README 冒頭に本番デモリンクがある

## 確認コマンド例

デプロイ後，次を確認する．

```bash
curl -L https://kentakom1213.github.io/graph-editor/ | grep -i "canonical"
curl -L https://kentakom1213.github.io/graph-editor/ | grep -i "description"
curl -L https://kentakom1213.github.io/graph-editor/ | grep -i "noindex"
curl -L https://kentakom1213.github.io/graph-editor/sitemap.xml
curl -L https://kentakom1213.github.io/graph-editor/robots.txt
```

本番ページで `noindex` が出てこないことを必ず確認する．

```bash
curl -L https://kentakom1213.github.io/graph-editor/ | grep -i "noindex"
```

このコマンドで何も出なければよい．

## 優先度

最優先は次の 3 点である．

1. 本番ページに `noindex` を入れない
2. canonical / OGP / sitemap を本番 URL に統一する
3. 静的説明文の重複を減らす

この 3 点ができていれば，PR-37 の SEO 改善としては十分に良い状態になる．
