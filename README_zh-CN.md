<p align="center">
  <img src="tauri-app/src-tauri/icons/icon-256x256.png" width="72" height="72" alt="Video Downloader Light icon">
</p>

<h1 align="center">Video Downloader Light</h1>

<p align="center">
  <strong>一款轻量级 Windows 桌面下载工具，将实际下载工作保留在用户电脑本地执行。</strong>
</p>

<p align="center">
  <a href="README.md">English</a> ·
  <a href="README_ko.md">한국어</a> ·
  <a href="README_zh-CN.md"><b>简体中文</b></a> ·
  <a href="README_ja.md">日本語</a>
</p>

<p align="center">
  <a href="https://github.com/Aminoragit/Video_Downloader_Light/releases/latest">
    <img src="https://img.shields.io/badge/下载-Latest%20Release-2459C6?style=for-the-badge" alt="下载最新版本">
  </a>
  <img src="https://img.shields.io/badge/Platform-Windows-207965?style=for-the-badge" alt="Windows">
  <img src="https://img.shields.io/badge/App-Tauri%202-172331?style=for-the-badge" alt="Tauri 2">
</p>

## 概览

Video Downloader Light 是一款本地优先的桌面应用。服务器不会代替用户下载或保存媒体文件。用户安装应用后，输入公开视频 URL，选择保存和质量选项，然后由本机通过 `yt-dlp`、`ffmpeg` 和 `aria2c` 执行下载。

安装包保持轻量。必要的运行文件会在首次启动时从官方来源下载，并存放在用户本地应用数据目录中。

## 下载

- 最新发布页：[GitHub Releases](https://github.com/Aminoragit/Video_Downloader_Light/releases/latest)
- 安装包直接链接：[Video Downloader Light_0.1.0_x64-setup.exe](https://github.com/Aminoragit/Video_Downloader_Light/releases/latest/download/Video%20Downloader%20Light_0.1.0_x64-setup.exe)

## 主要功能

- 三页式界面：下载、历史记录、捐赠。
- URL 输入、保存目录选择、视频/音频/All 模式、视频质量和音频质量设置。
- 进度区域显示缩略图、标题、时长、预估大小、进度条和日志。
- 本地下载历史。"重新下载请求"只会把旧 URL 填入输入框，不会自动开始下载。
- 首次运行自动安装 `yt-dlp`、`ffmpeg`、`aria2c`，并显示实时进度。
- 保留广告区域和捐赠页面，但不把下载处理迁移到服务器。
- 启动前显示法律提示，明确版权、DRM、再分发和平台条款风险。

## 使用方法

1. 启动应用并接受法律提示。
2. 应用检查必要组件，并安装缺失文件。
3. 输入受支持的公开视频 URL。
4. 选择保存目录、下载类型、视频质量和音频质量。
5. 点击下载。
6. 在进度页面查看状态，并在历史记录页面查看结果。

## 隐私模型

- 媒体下载在用户电脑本地执行。
- 应用不会把下载后的媒体上传到后端服务器。
- 不会自动收集浏览器 Cookie、登录会话或认证令牌。
- 设置和历史记录保存在本地。

## 法律与政策提示

本项目仅用于帮助用户在合法授权范围内进行个人归档。它不用于 DRM 绕过、付费 OTT 内容复制、访问控制绕过、再分发、转售或商业使用。

用户需要自行遵守版权法、本地法规以及各平台的服务条款。

## 从源码构建

要求：

- Windows
- Node.js 和 npm
- Rust toolchain
- WebView2 Runtime

```powershell
# 构建 Tauri 安装包
.\build-tauri.ps1

# 或直接构建
cd tauri-app
npm install
npm run build
```

生成的 NSIS 安装包路径：

```text
tauri-app/src-tauri/target/release/bundle/nsis/Video Downloader Light_0.1.0_x64-setup.exe
```

## 项目结构

```text
tauri-app/
  ui/                 前端 HTML、CSS、JavaScript
  src-tauri/          Rust 后端、Tauri 配置、图标和安装包构建
build-tauri.ps1       Windows 构建脚本
```

## 支持

- 捐赠：[Buy Me a Coffee](https://www.buymeacoffee.com/aminora)
- 问题反馈：[GitHub Issues](https://github.com/Aminoragit/Video_Downloader_Light/issues)

## 许可证

当前仓库尚未包含独立的许可证文件。若要重新分发或在其他项目中复用源码，请先添加正式许可证。
