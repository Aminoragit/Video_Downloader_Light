<p align="center">
  <img src="tauri-app/src-tauri/icons/icon-256x256.png" width="72" height="72" alt="Video Downloader Light icon">
</p>

<h1 align="center">Video Downloader Light</h1>

<p align="center">
  <strong>A lightweight Windows desktop downloader that keeps the download work on the user's PC.</strong>
</p>

<p align="center">
  <a href="README.md"><b>English</b></a> ·
  <a href="README_ko.md">한국어</a> ·
  <a href="README_zh-CN.md">简体中文</a> ·
  <a href="README_ja.md">日本語</a>
</p>

<p align="center">
  <a href="https://github.com/Aminoragit/Video_Downloader_Light/releases/latest">
    <img src="https://img.shields.io/badge/Download-Latest%20Release-2459C6?style=for-the-badge" alt="Download latest release">
  </a>
  <img src="https://img.shields.io/badge/Platform-Windows-207965?style=for-the-badge" alt="Windows">
  <img src="https://img.shields.io/badge/App-Tauri%202-172331?style=for-the-badge" alt="Tauri 2">
</p>

## Overview

Video Downloader Light is a local-first desktop app. The server does not process or host downloaded media. Users install the app, paste a public video URL, choose the output options, and the app performs the work locally with `yt-dlp`, `ffmpeg`, and `aria2c`.

The installer is intentionally small. Required binaries are downloaded from their official sources on first launch and stored under the user's local app data directory.

## Download

- Latest release page: [GitHub Releases](https://github.com/Aminoragit/Video_Downloader_Light/releases/latest)
- Direct installer link: [Video Downloader Light_0.1.0_x64-setup.exe](https://github.com/Aminoragit/Video_Downloader_Light/releases/latest/download/Video%20Downloader%20Light_0.1.0_x64-setup.exe)

## Key Features

- Three-page UI: Download, History, Donation.
- URL input with save-folder picker, video/audio/all mode, video quality, and audio quality.
- Real-time progress area with thumbnail, title, duration, estimated size, progress bar, and logs.
- Local download history with "request again" behavior that only fills the URL. The user must click download manually.
- Automatic first-run setup for `yt-dlp`, `ffmpeg`, and `aria2c` with progress feedback.
- Affiliate ad area and donation page without moving download processing to a server.
- Legal notice before use, with explicit warnings about copyright, DRM, redistribution, and platform terms.

## How It Works

1. Launch the app and accept the legal notice.
2. The app checks required binaries and downloads missing components.
3. Paste a supported public video URL.
4. Choose save folder, download type, video quality, and audio quality.
5. Click Download.
6. Watch progress and review the result in the history page.

## Privacy Model

- Media download work runs on the user's PC.
- The app does not upload downloaded media to a service backend.
- Browser cookies, login sessions, and authentication tokens are not automatically collected.
- Settings and history are stored locally.

## Legal And Policy Notice

This project is intended for lawful personal archiving of content that the user is authorized to access and save. It is not designed for DRM circumvention, paid OTT copying, access-control bypass, redistribution, resale, or commercial reuse of downloaded media.

Users are responsible for following copyright law, local regulations, and each platform's terms of service.

## Build From Source

Requirements:

- Windows
- Node.js and npm
- Rust toolchain
- WebView2 Runtime

```powershell
# Build the Tauri installer
.\build-tauri.ps1

# Or build directly
cd tauri-app
npm install
npm run build
```

The generated NSIS installer is created at:

```text
tauri-app/src-tauri/target/release/bundle/nsis/Video Downloader Light_0.1.0_x64-setup.exe
```

## Project Structure

```text
tauri-app/
  ui/                 Frontend HTML, CSS, and JavaScript
  src-tauri/          Rust backend, Tauri config, icons, installer build
build-tauri.ps1       Windows build helper
```

## Support

- Donation: [Buy Me a Coffee](https://www.buymeacoffee.com/aminora)
- Issues: [GitHub Issues](https://github.com/Aminoragit/Video_Downloader_Light/issues)

## License

This repository does not currently include a dedicated license file. Add a formal license before redistributing forks or reusing the source in another project.
