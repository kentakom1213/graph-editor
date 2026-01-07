# 画像出力機能の実装方針

この文書は，現在の構成に合わせて画像出力（PNG など）を追加する際の方針を整理する．

## 要求

- PNG / SVG でのエクスポート

## 変更箇所

- UI 追加: `src/components/edit_menu.rs`
  - 既存の Edit メニュー内に「Export Image」ボタンを追加する．
- 描画取得: `src/components/central_panel.rs`
  - 描画と同じロジックで `egui::ColorImage` を作る関数を切り出す（UI 描画と画像生成を共有）．
- 書き出し処理: `src/app.rs`
  - 画像生成処理の呼び出しと保存ダイアログ連携の入口を追加する．

## ボタン配置

- Edit メニュー内の「Reset Colors」または「Clear All」付近に配置する．
- ラベル例: `Export Image`
- Export 中は多重クリックを防ぐため，必要ならボタンを一時的に disable にする．

## ライブラリ候補

- `egui` の `egui::ColorImage` と `image` クレートで PNG へ変換・保存する．
- パス選択に `rfd`（Rust File Dialog）を使う．

例:

- `image` クレート: `image::RgbaImage::from_raw` + `save` で出力
- `rfd` クレート: `rfd::FileDialog::new().save_file()`

## 実装の流れ（概要）

1. 既存の描画処理を「描画コマンド生成」と「画面描画」に分離する．
2. 描画コマンドから `egui::ColorImage` を生成する関数を追加する．
3. Export ボタン押下で保存ダイアログを開き，画像を保存する．

## 注意点

- 画面サイズに依存するため，エクスポート時の解像度（例: 1024x1024）を指定できるようにするのが望ましい．
- UI での座標変換（アフィン変換）を適用した状態で書き出す必要がある．

## wasm 対応の指針

- そのままの実装は wasm では使えない
  - `rfd::FileDialog::save_file()` は wasm で同期ダイアログを出せないため非対応
  - ブラウザ上ではファイルパスに直接書き込めない（ユーザー操作のダウンロードになる）
- 対応方法
  - 画像生成（`egui::ColorImage` -> PNG bytes）は同じロジックで OK
  - 保存は `rfd::AsyncFileDialog::new().save_file().await` を使う
    - wasm ではフィルタは無視され，`save_file` は即座に `FileHandle` を返す
    - `FileHandle::write(bytes).await` でブラウザの保存ダイアログが出る
  - async が必要なので `wasm_bindgen_futures::spawn_local` 等で駆動する
  - 代替案として `web_sys` で `Blob` + `URL.createObjectURL` + `<a download>` を生成してダウンロードさせる方法もある
