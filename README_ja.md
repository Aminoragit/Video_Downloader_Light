<p align="center">
  <img src="tauri-app/src-tauri/icons/icon-256x256.png" width="72" height="72" alt="Video Downloader Light icon">
</p>

<h1 align="center">Video Downloader Light</h1>

<p align="center">
  <strong>ダウンロード処理をユーザーのPC上で実行する、軽量な Windows デスクトップアプリです。</strong>
</p>

<p align="center">
  <a href="README.md">English</a> ·
  <a href="README_ko.md">한국어</a> ·
  <a href="README_zh-CN.md">简体中文</a> ·
  <a href="README_ja.md"><b>日本語</b></a>
</p>

<p align="center">
  <a href="https://github.com/Aminoragit/Video_Downloader_Light/releases/latest">
    <img src="https://img.shields.io/badge/Download-Latest%20Release-2459C6?style=for-the-badge" alt="最新リリースをダウンロード">
  </a>
  <img src="https://img.shields.io/badge/Platform-Windows-207965?style=for-the-badge" alt="Windows">
  <img src="https://img.shields.io/badge/App-Tauri%202-172331?style=for-the-badge" alt="Tauri 2">
</p>

## 概要

Video Downloader Light はローカルファーストのデスクトップアプリです。サーバーがメディアを代理ダウンロードしたり保存したりする設計ではありません。ユーザーはアプリをインストールし、公開動画の URL を入力し、保存先や品質を選択して、`yt-dlp`、`ffmpeg`、`aria2c` による処理を自分の PC 上で実行します。

インストーラーは小さく保たれています。必要な実行ファイルは初回起動時に公式配布元から取得され、ユーザーのローカルアプリデータ配下に保存されます。

## ダウンロード

- 最新リリースページ：[GitHub Releases](https://github.com/Aminoragit/Video_Downloader_Light/releases/latest)
- インストーラー直接リンク：[Video Downloader Light_0.1.0_x64-setup.exe](https://github.com/Aminoragit/Video_Downloader_Light/releases/latest/download/Video.Downloader.Light_0.1.0_x64-setup.exe)

## 主な機能

- 3ページ構成の UI：ダウンロード、履歴、ドネーション。
- URL 入力、保存先選択、動画/音声/All、動画品質、音声品質の設定。
- 進行状況エリアにサムネイル、タイトル、長さ、推定サイズ、進捗バー、ログを表示。
- ローカル履歴を表示。"新しいダウンロード要求"は URL だけを入力欄に戻し、自動で開始しません。
- 初回起動時に `yt-dlp`、`ffmpeg`、`aria2c` を自動セットアップし、進捗を表示。
- サーバー側でダウンロード処理を行わず、広告エリアと寄付ページを維持。
- 利用前に法的注意を表示し、著作権、DRM、再配布、各プラットフォーム規約への注意を明示。

## 使い方

1. アプリを起動し、法的注意に同意します。
2. アプリが必要な実行ファイルを確認し、不足分をインストールします。
3. 対応する公開動画 URL を入力します。
4. 保存先、ダウンロード種類、動画品質、音声品質を選択します。
5. ダウンロードボタンを押します。
6. 進捗を確認し、完了後は履歴ページで記録を確認します。

## プライバシーモデル

- メディアのダウンロード処理はユーザーの PC 上で実行されます。
- ダウンロード済みメディアをバックエンドサーバーへアップロードしません。
- ブラウザー Cookie、ログインセッション、認証トークンを自動収集しません。
- 設定と履歴はローカルに保存されます。

## 法的およびポリシー上の注意

本プロジェクトは、ユーザーが合法的にアクセスし保存する権限を持つコンテンツを、個人的な範囲でアーカイブするための補助ツールです。DRM 回避、有料 OTT コンテンツの複製、アクセス制御の回避、再配布、販売、商用利用を目的としていません。

ユーザーは著作権法、地域の法令、各プラットフォームの利用規約を自分で確認し、遵守する必要があります。

## ソースからビルド

必要条件：

- Windows
- Node.js と npm
- Rust toolchain
- WebView2 Runtime

```powershell
# Tauri インストーラーをビルド
.\build-tauri.ps1

# または直接ビルド
cd tauri-app
npm install
npm run build
```

生成される NSIS インストーラーの場所：

```text
tauri-app/src-tauri/target/release/bundle/nsis/Video Downloader Light_0.1.0_x64-setup.exe
```

## プロジェクト構成

```text
tauri-app/
  ui/                 フロントエンド HTML、CSS、JavaScript
  src-tauri/          Rust バックエンド、Tauri 設定、アイコン、インストーラービルド
build-tauri.ps1       Windows ビルド補助スクリプト
```

## サポート

- 寄付：[Buy Me a Coffee](https://www.buymeacoffee.com/aminora)
- 問い合わせ：[GitHub Issues](https://github.com/Aminoragit/Video_Downloader_Light/issues)

## ライセンス

現在、このリポジトリには独立したライセンスファイルが含まれていません。ソースの再配布や別プロジェクトでの再利用を行う前に、正式なライセンスを追加してください。
