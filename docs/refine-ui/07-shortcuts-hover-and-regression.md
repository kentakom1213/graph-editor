# Step 7: ショートカット・ホバー制御・回帰確認

## 目的

見た目の移行だけでなく，キャンバス操作抑止と既存機能維持を最後に検証する．

## 対象ファイル

- `src/app.rs`
- `src/components/top_panel.rs`
- `src/components/tool_bar.rs`
- `src/components/inspector_panel.rs`
- キーボード入力を扱う関連箇所

## 確認項目

### 1. ショートカット維持

- `Esc`: Normal
- `V`: Add Vertex
- `E`: Add Edge
- `C`: Colorize
- `D`: Delete
- `Shift + D`: Directed / Undirected 切替
- `1`: 0-indexed / 1-indexed 切替
- `A`: Animate 切替

### 2. 入力フォーカス中の抑止

- `code_editor` にフォーカスがある間はショートカットが発火しない
- `input_has_focus` が UI 上の実際のフォーカス状態と一致している

### 3. ホバー中のキャンバス操作抑止

- `top_panel` 上で頂点追加されない
- `tool_bar` 上で頂点追加されない
- `inspector_panel` 上で頂点追加されない
- 右パネル内の editor 操作でキャンバス入力が混線しない

### 4. 既存機能の回帰確認

- 頂点追加
- 頂点ドラッグ
- 辺追加
- 頂点削除
- 辺削除
- 頂点色変更
- 辺色変更
- `0-indexed` / `1-indexed` 切替
- `Directed` / `Undirected` 切替
- 頂点番号表示切替
- アニメーション切替
- `Copy`
- `Apply`
- `Complement`
- `Revert Edge`
- `Reset Colors`
- `Clear All`
- `PNG` / `SVG` Export
- エラーモーダル表示
- Clear All 確認モーダル表示

## 実装メモ

- ここは新規 UI 実装ではなく，抜け漏れ確認の最終段と位置付ける
- 手動確認項目が多いので，必要なら簡易チェックリストとして消し込みながら進める

## 完了条件

- 固定パネル化後も主要ショートカットが維持されている
- キャンバスとパネルの入力境界が壊れていない
- `00-overall_plan.md` に挙がっている既存機能の維持条件を満たしている
