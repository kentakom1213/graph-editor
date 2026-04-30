# Step 5: 上部バーと下部ステータスバーの整理

## 目的

旧タブバーを役割の薄いヘッダーへ置き換え，フッターを常設ステータスバーへ作り替える．

## 対象ファイル

- `src/components/top_panel.rs`
- `src/components/footer.rs`
- `src/components/mod.rs`
- `src/config.rs`

## 実装内容

### 1. 上部パネルを簡素化する

表示内容:

```text
Graph Editor                              [Export] [Settings]
```

仕様:

- `Export` は `app.request_export_image(ctx)` を呼ぶ
- `Settings` は当面ダミーボタンでよい
- 旧 `Menu / Color / Input` toggle は削除する

### 2. 下部をステータスバー化する

`egui::Area` ベースではなく `TopBottomPanel::bottom` に変更する．

表示候補:

- 現在の `edit_mode`
- `0-indexed` / `1-indexed`
- `Directed` / `Undirected`
- 頂点数
- 辺数
- バージョン表記
- GitHub リンク

### 3. 件数表示の数え方を決める

- 削除済み要素を含めるか除外するかを明示する
- UI 表示としては `is_deleted` 除外が自然ならその方針に揃える

## 実装メモ

- `draw_footer` の名前は据え置きでもよい
- 余裕があれば `status_bar.rs` へ改名する
- ステータスバーは左に状態，右にアプリ情報を寄せると読みやすい

## 完了条件

- 上部から旧表示切替 UI が消えている
- 下部に現在状態が一行で読めるステータスバーがある
- バージョン表示と GitHub リンクが維持されている
