# Video Downloader Light Tauri App

This directory contains the lightweight Tauri/WebView2 desktop client.

## Notes

- The installer does not bundle `yt-dlp`, `ffmpeg`, or `aria2c`.
- Required binaries are downloaded on first launch into `%LOCALAPPDATA%\VideoDownloaderLight\bin`.
- The UI lives in `ui/` and the Rust backend lives in `src-tauri/`.
- Build output is generated under `src-tauri/target/`, which is ignored.

## Build

```powershell
cd ..
.\build-tauri.ps1
```

The NSIS installer is generated under:

```text
tauri-app\src-tauri\target\release\bundle\nsis
```
