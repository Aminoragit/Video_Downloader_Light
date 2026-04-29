<p align="center">
  <img src="tauri-app/src-tauri/icons/icon-256x256.png" width="72" height="72" alt="Video Downloader Light icon">
</p>

<h1 align="center">Video Downloader Light</h1>

<p align="center">
  <strong>다운로드 처리는 사용자 PC에서 실행하고, 배포 파일은 작게 유지하는 Windows 데스크톱 앱입니다.</strong>
</p>

<p align="center">
  <a href="README.md">English</a> ·
  <a href="README_ko.md"><b>한국어</b></a> ·
  <a href="README_zh-CN.md">简体中文</a> ·
  <a href="README_ja.md">日本語</a>
</p>

<p align="center">
  <a href="https://github.com/weallnoob/Video_Downloader/releases/latest">
    <img src="https://img.shields.io/badge/다운로드-Latest%20Release-2459C6?style=for-the-badge" alt="최신 릴리스 다운로드">
  </a>
  <img src="https://img.shields.io/badge/Platform-Windows-207965?style=for-the-badge" alt="Windows">
  <img src="https://img.shields.io/badge/App-Tauri%202-172331?style=for-the-badge" alt="Tauri 2">
</p>

## 개요

Video Downloader Light는 로컬 우선 방식의 데스크톱 앱입니다. 서버가 영상 파일을 대신 다운로드하거나 보관하지 않습니다. 사용자는 앱을 설치한 뒤 공개 영상 URL을 입력하고, 저장 옵션을 선택하고, `yt-dlp`, `ffmpeg`, `aria2c` 기반 다운로드를 자신의 PC에서 실행합니다.

설치 파일은 작게 유지합니다. 필수 실행 파일은 첫 실행 시 공식 출처에서 내려받고 사용자 로컬 앱 데이터 폴더에 저장합니다.

## 다운로드

- 최신 릴리스 페이지: [GitHub Releases](https://github.com/weallnoob/Video_Downloader/releases/latest)
- 설치 파일 직접 링크: [Video Downloader Light_0.1.0_x64-setup.exe](https://github.com/weallnoob/Video_Downloader/releases/latest/download/Video%20Downloader%20Light_0.1.0_x64-setup.exe)

## 주요 기능

- 3개 페이지 UI: 다운로드, 이력, 도네이션.
- URL 입력, 저장 폴더 선택, 비디오/오디오/All 선택, 비디오 품질, 오디오 품질 설정.
- 진행 카드에서 섬네일, 제목, 길이, 예상 용량, 프로그래스바, 로그 표시.
- 이전 다운로드 이력 제공. "새로 다운로드 요청"은 URL만 입력칸에 채우며, 실제 다운로드는 사용자가 버튼을 눌러야 시작됩니다.
- 첫 실행 시 `yt-dlp`, `ffmpeg`, `aria2c` 자동 설치와 실시간 진행 표시.
- 서버 다운로드 처리 없이 앱 내부 광고 영역과 후원 페이지 유지.
- 사용 전 법적 고지 표시. 저작권, DRM, 재배포, 플랫폼 약관 위반에 대한 책임을 명확히 안내합니다.

## 사용 방법

1. 앱을 실행하고 법적 고지에 동의합니다.
2. 앱이 필수 실행 파일을 확인하고 누락된 구성요소를 설치합니다.
3. 지원되는 공개 영상 URL을 입력합니다.
4. 저장 폴더, 다운로드 유형, 비디오 품질, 오디오 품질을 선택합니다.
5. 다운로드 버튼을 누릅니다.
6. 진행 상황을 확인하고, 완료 후 이력 페이지에서 기록을 확인합니다.

## 개인정보 처리 방향

- 영상 다운로드 처리는 사용자 PC에서 실행됩니다.
- 다운로드된 미디어 파일을 서버로 업로드하지 않습니다.
- 브라우저 쿠키, 로그인 세션, 인증 토큰을 자동 수집하지 않습니다.
- 설정과 다운로드 이력은 로컬에 저장됩니다.

## 법적 고지

본 프로젝트는 사용자가 합법적으로 접근하고 저장할 권한이 있는 콘텐츠를 개인적 범위에서 보관하도록 돕기 위한 도구입니다. DRM 우회, 유료 OTT 복제, 접근 통제 우회, 재배포, 판매, 상업적 이용을 목적으로 하지 않습니다.

사용자는 저작권법, 거주 지역의 법령, 각 플랫폼의 이용약관을 직접 확인하고 준수해야 합니다.

## 소스에서 빌드

필요 조건:

- Windows
- Node.js 및 npm
- Rust toolchain
- WebView2 Runtime

```powershell
# Tauri 설치 파일 빌드
.\build-tauri.ps1

# 직접 빌드
cd tauri-app
npm install
npm run build
```

생성되는 NSIS 설치 파일 위치:

```text
tauri-app/src-tauri/target/release/bundle/nsis/Video Downloader Light_0.1.0_x64-setup.exe
```

## 프로젝트 구조

```text
tauri-app/
  ui/                 프론트엔드 HTML, CSS, JavaScript
  src-tauri/          Rust 백엔드, Tauri 설정, 아이콘, 설치 파일 빌드
build-tauri.ps1       Windows 빌드 헬퍼
```

## 후원 및 문의

- 후원: [Buy Me a Coffee](https://www.buymeacoffee.com/aminora)
- 이슈: [GitHub Issues](https://github.com/weallnoob/Video_Downloader/issues)

## 라이선스

현재 저장소에는 별도 라이선스 파일이 포함되어 있지 않습니다. 소스 재배포 또는 포크 재사용 전에는 정식 라이선스를 추가하는 것이 필요합니다.
