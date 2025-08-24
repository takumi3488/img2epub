# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## プロジェクト概要

img2epubは、画像ファイルをEPUB形式の電子書籍に変換するRustベースのコマンドラインツールです。複数の画像ファイルを1つのEPUBファイルにまとめ、メタデータの設定や読書方向（LTR/RTL）の指定が可能です。

## ビルドとテストコマンド

```bash
# ビルド
cargo build --release

# リリースビルドと実行
cargo run --release -- ./images test.epub

# Clippyで静的解析
cargo clippy

# コードフォーマット
cargo fmt

# Taskfileを使った統合テスト（EPUBCheckを含む）
task test
```

## アーキテクチャ

### コアモジュール構造

- `src/lib.rs`: メインのimg2epub関数。画像処理、EPUB構造の生成、メタデータ管理を統括
- `src/epub/converter.rs`: EPUBファイル構造の生成（OPF、NAVファイル等）とZIP圧縮
- `src/epub/images.rs`: 画像ファイルの処理（ソート、パディング、WebP変換）
- `src/bin/img2epub.rs`: CLIエントリーポイント
- `src/bin/get_metadata.rs`: EPUBからメタデータを取得するユーティリティ

### EPUB生成フロー

1. メタデータの読み込み（metadata.jsonまたはCLI引数）
2. 画像ファイルをファイル名でソート
3. 最大幅・高さを計算し、全画像をパディング
4. WebP形式に変換して一時ディレクトリ（/tmp/epub-UUID）に保存
5. EPUB構造ファイル（OPF、NAV、HTML）を生成
6. ZIP圧縮してEPUBファイルを作成
7. 一時ディレクトリを削除

### 重要な設計判断

- 画像は全てWebP形式に変換される
- 全画像は最大サイズに合わせてパディングされる
- RTL（右から左）とLTR（左から右）の読書方向をサポート
- オプションで冒頭に空白ページを追加可能
- EPUBCheckを使用してEPUB仕様への準拠を検証