use serde::{Deserialize, Serialize};
use std::{
    env, fs,
    fs::File,
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex,
    },
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use tauri::{AppHandle, Emitter, Manager};
use url::Url;

const YTDLP_URL: &str = "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp.exe";
const ARIA2_ZIP_URL: &str = "https://github.com/aria2/aria2/releases/download/release-1.37.0/aria2-1.37.0-win-64bit-build1.zip";
const FFMPEG_ZIP_URL: &str = "https://www.gyan.dev/ffmpeg/builds/ffmpeg-release-essentials.zip";

static SETTINGS_LOCK: Mutex<()> = Mutex::new(());
static HISTORY_LOCK: Mutex<()> = Mutex::new(());
static NEXT_ID: AtomicU64 = AtomicU64::new(1);

const ALLOW_DOMAINS: &[&str] = &[
    "youtube.com",
    "youtu.be",
    "vimeo.com",
    "dailymotion.com",
    "dai.ly",
    "facebook.com",
    "fb.watch",
    "instagram.com",
    "x.com",
    "twitter.com",
    "tiktok.com",
    "vm.tiktok.com",
    "twitch.tv",
    "kick.com",
    "trovo.live",
    "rumble.com",
    "bilibili.com",
    "b23.tv",
    "youku.com",
    "iq.com",
    "iqiyi.com",
    "v.qq.com",
    "qq.com",
    "nicovideo.jp",
    "nico.ms",
    "vk.com",
    "vkvideo.ru",
    "rutube.ru",
    "odysee.com",
    "d.tube",
    "dtube.network",
    "peertube.tv",
    "bitchute.com",
    "ted.com",
    "coursera.org",
    "udemy.com",
    "linkedin.com",
    "snapchat.com",
    "triller.co",
    "triller.tv",
    "tubitv.com",
    "pluto.tv",
    "crackle.com",
    "sonycrackle.com",
    "afreecatv.com",
    "chzzk.naver.com",
    "tv.naver.com",
    "tv.kakao.com",
    "douyin.com",
    "acfun.cn",
    "mgtv.com",
    "mangotv.com",
    "bbc.co.uk",
    "bbc.com",
    "itv.com",
    "channel4.com",
    "channel5.com",
    "ardmediathek.de",
    "ard.de",
    "zdf.de",
    "france.tv",
    "nhk.or.jp",
    "nhk.jp",
    "aljazeera.com",
    "bloomberg.com",
    "cnn.com",
    "foxnation.com",
    "viu.com",
    "iflix.com",
    "hoichoi.tv",
    "altbalaji.com",
    "erosnow.com",
    "filmrise.com",
    "hayu.com",
    "magellantv.com",
    "bongobd.com",
    "stageit.com",
];

const BLOCK_DOMAINS: &[&str] = &[
    "netflix.com",
    "hulu.com",
    "primevideo.com",
    "amazon.com",
    "disneyplus.com",
    "tv.apple.com",
    "apple.com",
    "max.com",
    "hbomax.com",
    "paramountplus.com",
    "peacocktv.com",
    "crunchyroll.com",
    "rakuten.tv",
    "zee5.com",
    "hotstar.com",
    "viki.com",
    "wetv.vip",
    "mxplayer.in",
    "sonyliv.com",
    "discoveryplus.com",
    "espn.com",
    "shudder.com",
    "gaia.com",
    "nebula.tv",
    "curiositystream.com",
    "kanopy.com",
    "plex.tv",
    "redbox.com",
    "vudu.com",
    "fandango.com",
    "fubo.tv",
    "sling.com",
    "tv.youtube.com",
];

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ComponentStatus {
    installed: bool,
    path: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AppState {
    ytdlp: ComponentStatus,
    ffmpeg: ComponentStatus,
    aria2c: ComponentStatus,
    default_save_dir: String,
    settings: UserSettings,
    history: Vec<HistoryRecord>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
struct UserSettings {
    save_dir: String,
    mode: String,
    quality: String,
    audio_quality: String,
    audio_format: String,
    legal_accepted: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SettingsInput {
    save_dir: String,
    mode: String,
    quality: String,
    audio_quality: String,
    audio_format: String,
}

impl Default for UserSettings {
    fn default() -> Self {
        Self {
            save_dir: default_save_dir().to_string_lossy().into_owned(),
            mode: "all".to_string(),
            quality: "best".to_string(),
            audio_quality: "best".to_string(),
            audio_format: "original".to_string(),
            legal_accepted: false,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct HistoryRecord {
    id: String,
    url: String,
    requested_at: String,
    status: String,
    output_dir: String,
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct DownloadRequest {
    url: String,
    save_dir: String,
    mode: String,
    quality: String,
    audio_quality: String,
}

#[derive(Serialize, Clone)]
struct ProgressPayload {
    percent: f64,
    message: String,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct InstallProgressPayload {
    name: String,
    percent: f64,
    message: String,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct DownloadMetadataPayload {
    history_id: String,
    title: String,
    duration: String,
    size: String,
    thumbnail: String,
    size_approximate: bool,
}

fn app_data_dir() -> PathBuf {
    env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
        .join("VideoDownloaderLight")
}

fn bin_dir() -> PathBuf {
    app_data_dir().join("bin")
}

fn settings_path() -> PathBuf {
    app_data_dir().join("settings.json")
}

fn history_path() -> PathBuf {
    app_data_dir().join("history.json")
}

fn default_save_dir() -> PathBuf {
    env::var_os("USERPROFILE")
        .map(PathBuf::from)
        .map(|home| home.join("Downloads").join("VideoDownloader"))
        .unwrap_or_else(|| app_data_dir().join("downloads"))
}

fn now_unix_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

fn now_unix_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0)
}

fn atomic_write(path: &Path, payload: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| format!("저장 폴더 생성 실패: {err}"))?;
    }

    let temp_path = path.with_extension("tmp");
    {
        let mut file =
            File::create(&temp_path).map_err(|err| format!("임시 파일 생성 실패: {err}"))?;
        file.write_all(payload.as_bytes())
            .map_err(|err| format!("임시 파일 쓰기 실패: {err}"))?;
        file.sync_all()
            .map_err(|err| format!("임시 파일 동기화 실패: {err}"))?;
    }

    fs::rename(&temp_path, path).map_err(|err| format!("파일 교체 실패: {err}"))
}

fn read_settings_file() -> UserSettings {
    fs::read_to_string(settings_path())
        .ok()
        .and_then(|raw| serde_json::from_str::<UserSettings>(&raw).ok())
        .unwrap_or_default()
}

fn write_settings_file(settings: &UserSettings) -> Result<(), String> {
    let payload =
        serde_json::to_string_pretty(settings).map_err(|err| format!("설정 직렬화 실패: {err}"))?;
    atomic_write(&settings_path(), &payload)
}

fn read_settings() -> UserSettings {
    let _guard = SETTINGS_LOCK.lock().ok();
    read_settings_file()
}

fn read_history_file() -> Vec<HistoryRecord> {
    fs::read_to_string(history_path())
        .ok()
        .and_then(|raw| serde_json::from_str::<Vec<HistoryRecord>>(&raw).ok())
        .unwrap_or_default()
}

fn write_history_file(records: &[HistoryRecord]) -> Result<(), String> {
    let payload =
        serde_json::to_string_pretty(records).map_err(|err| format!("기록 직렬화 실패: {err}"))?;
    atomic_write(&history_path(), &payload)
}

fn read_history() -> Vec<HistoryRecord> {
    let _guard = HISTORY_LOCK.lock().ok();
    read_history_file()
}

fn append_history(url: &str, output_dir: &Path) -> Result<String, String> {
    let _guard = HISTORY_LOCK
        .lock()
        .map_err(|_| "기록 저장 잠금을 획득하지 못했습니다.".to_string())?;
    let id = format!(
        "{}-{}",
        now_unix_millis(),
        NEXT_ID.fetch_add(1, Ordering::Relaxed)
    );
    let record = HistoryRecord {
        id: id.clone(),
        url: url.to_string(),
        requested_at: format!("{}", now_unix_seconds()),
        status: "진행 중".to_string(),
        output_dir: output_dir.to_string_lossy().into_owned(),
    };
    let mut records = read_history_file();
    records.insert(0, record);
    records.truncate(300);
    write_history_file(&records)?;
    Ok(id)
}

fn update_history_status(id: &str, status: &str) {
    let Ok(_guard) = HISTORY_LOCK.lock() else {
        return;
    };
    let mut records = read_history_file();
    if let Some(record) = records.iter_mut().find(|record| record.id == id) {
        record.status = status.to_string();
        let _ = write_history_file(&records);
    }
}

fn component_path(name: &str) -> Result<PathBuf, String> {
    let filename = match name {
        "ytdlp" => "yt-dlp.exe",
        "ffmpeg" => "ffmpeg.exe",
        "aria2c" => "aria2c.exe",
        _ => return Err(format!("알 수 없는 구성요소입니다: {name}")),
    };
    Ok(bin_dir().join(filename))
}

fn component_status(name: &str) -> ComponentStatus {
    let path = component_path(name).unwrap_or_else(|_| bin_dir().join(name));
    ComponentStatus {
        installed: validate_component_binary(name, &path).is_ok(),
        path: path.to_string_lossy().into_owned(),
    }
}

#[tauri::command]
fn get_state() -> AppState {
    AppState {
        ytdlp: component_status("ytdlp"),
        ffmpeg: component_status("ffmpeg"),
        aria2c: component_status("aria2c"),
        default_save_dir: default_save_dir().to_string_lossy().into_owned(),
        settings: read_settings(),
        history: read_history(),
    }
}

#[tauri::command]
fn save_settings(settings: SettingsInput) -> Result<(), String> {
    let _guard = SETTINGS_LOCK
        .lock()
        .map_err(|_| "설정 저장 잠금을 획득하지 못했습니다.".to_string())?;
    let mut current = read_settings_file();
    current.save_dir = settings.save_dir;
    current.mode = settings.mode;
    current.quality = settings.quality;
    current.audio_quality = settings.audio_quality;
    current.audio_format = settings.audio_format;
    write_settings_file(&current)
}

#[tauri::command]
fn accept_legal() -> Result<(), String> {
    let _guard = SETTINGS_LOCK
        .lock()
        .map_err(|_| "설정 저장 잠금을 획득하지 못했습니다.".to_string())?;
    let mut settings = read_settings_file();
    settings.legal_accepted = true;
    write_settings_file(&settings)
}

#[tauri::command]
fn delete_history_record(id: String) -> Result<(), String> {
    let _guard = HISTORY_LOCK
        .lock()
        .map_err(|_| "기록 저장 잠금을 획득하지 못했습니다.".to_string())?;
    let mut records = read_history_file();
    records.retain(|record| record.id != id);
    write_history_file(&records)
}

#[tauri::command]
fn clear_history() -> Result<(), String> {
    let _guard = HISTORY_LOCK
        .lock()
        .map_err(|_| "기록 저장 잠금을 획득하지 못했습니다.".to_string())?;
    write_history_file(&[])
}

fn ps_quote(value: &Path) -> String {
    value.to_string_lossy().replace('\'', "''")
}

fn run_powershell_raw(script: &str) -> Result<String, String> {
    let script = format!(
        "[Console]::OutputEncoding = New-Object System.Text.UTF8Encoding $false; \
         $OutputEncoding = [Console]::OutputEncoding; {script}"
    );
    let output = Command::new("powershell.exe")
        .args([
            "-NoProfile",
            "-Sta",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            &script,
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|err| format!("PowerShell 실행 실패: {err}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    if !output.status.success() {
        return Err(if stderr.is_empty() { stdout } else { stderr });
    }
    Ok(stdout)
}

fn emit_install_progress(app: &AppHandle, name: &str, percent: f64, message: &str) {
    let _ = app.emit(
        "install-progress",
        InstallProgressPayload {
            name: name.to_string(),
            percent: percent.clamp(0.0, 100.0),
            message: message.to_string(),
        },
    );
}

fn parse_install_progress(line: &str) -> Option<(f64, String)> {
    let payload = line.strip_prefix("__VDL_INSTALL_PROGRESS__|")?;
    let mut parts = payload.splitn(2, '|');
    let percent = parts.next()?.trim().parse::<f64>().ok()?;
    let message = parts.next().unwrap_or("").trim().to_string();
    Some((percent, message))
}

fn run_install_powershell(app: &AppHandle, name: &str, script: &str) -> Result<String, String> {
    let script = format!(
        "[Console]::OutputEncoding = New-Object System.Text.UTF8Encoding $false; \
         $OutputEncoding = [Console]::OutputEncoding; {script}"
    );
    let mut child = Command::new("powershell.exe")
        .args([
            "-NoProfile",
            "-Sta",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            &script,
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|err| format!("PowerShell 실행 실패: {err}"))?;

    let stderr_lines: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let stderr_handle = child.stderr.take().map(|stderr| {
        let stderr_lines = Arc::clone(&stderr_lines);
        thread::spawn(move || {
            for line in BufReader::new(stderr).lines().map_while(Result::ok) {
                let trimmed = line.trim().to_string();
                if trimmed.is_empty() {
                    continue;
                }
                if let Ok(mut lines) = stderr_lines.lock() {
                    if lines.len() >= 80 {
                        lines.remove(0);
                    }
                    lines.push(trimmed);
                }
            }
        })
    });

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| "PowerShell 출력 스트림을 읽을 수 없습니다.".to_string())?;
    let mut output_lines = Vec::new();
    for line in BufReader::new(stdout).lines().map_while(Result::ok) {
        let trimmed = line.trim().to_string();
        if trimmed.is_empty() {
            continue;
        }
        if let Some((percent, message)) = parse_install_progress(&trimmed) {
            emit_install_progress(app, name, percent, &message);
        } else {
            output_lines.push(trimmed);
        }
    }

    let status = child
        .wait()
        .map_err(|err| format!("PowerShell 종료 확인 실패: {err}"))?;
    if let Some(handle) = stderr_handle {
        let _ = handle.join();
    }

    let stderr = stderr_lines
        .lock()
        .map(|lines| lines.join("\n"))
        .unwrap_or_default();
    if !status.success() {
        let stdout = output_lines.join("\n");
        return Err(if !stderr.trim().is_empty() {
            stderr
        } else if !stdout.trim().is_empty() {
            stdout
        } else {
            format!("PowerShell 종료 코드: {status}")
        });
    }

    Ok(if output_lines.is_empty() {
        "완료".to_string()
    } else {
        output_lines.join("\n")
    })
}

#[tauri::command]
fn choose_download_dir(current: String) -> Result<Option<String>, String> {
    let initial = if current.trim().is_empty() {
        default_save_dir()
    } else {
        PathBuf::from(current.trim())
    };
    let initial_quoted = ps_quote(&initial);
    let script = format!(
        "$ErrorActionPreference='Stop'; \
         Add-Type -AssemblyName System.Windows.Forms; \
         $dialog = New-Object System.Windows.Forms.FolderBrowserDialog; \
         $dialog.Description = '저장 폴더 선택'; \
         $dialog.ShowNewFolderButton = $true; \
         if (Test-Path -LiteralPath '{initial_quoted}') {{ $dialog.SelectedPath = '{initial_quoted}' }}; \
         $result = $dialog.ShowDialog(); \
         if ($result -eq [System.Windows.Forms.DialogResult]::OK) {{ Write-Output $dialog.SelectedPath }}"
    );
    let selected = run_powershell_raw(&script)?;
    let selected = selected.trim();
    if selected.is_empty() {
        Ok(None)
    } else {
        Ok(Some(selected.to_string()))
    }
}

#[tauri::command]
async fn install_component(app: AppHandle, name: String) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || install_component_sync(app, name))
        .await
        .map_err(|err| format!("설치 작업 실행 실패: {err}"))?
}

fn install_component_sync(app: AppHandle, name: String) -> Result<String, String> {
    fs::create_dir_all(bin_dir()).map_err(|err| format!("bin 폴더 생성 실패: {err}"))?;
    let target = component_path(&name)?;
    let target_quoted = ps_quote(&target);
    let parent = target
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(bin_dir);
    let parent_quoted = ps_quote(&parent);

    emit_install_progress(&app, &name, 0.0, "설치 준비 중");
    let script = match name.as_str() {
        "ytdlp" => format!(
            "{download_helper} \
             $ErrorActionPreference='Stop'; \
             [Net.ServicePointManager]::SecurityProtocol=[Net.SecurityProtocolType]::Tls12; \
             New-Item -Path '{parent_quoted}' -ItemType Directory -Force | Out-Null; \
             $tmp = '{target_quoted}.download'; \
             try {{ \
                 if (Test-Path -LiteralPath $tmp) {{ Remove-Item -LiteralPath $tmp -Force }}; \
                 Save-VDLFile -Uri '{YTDLP_URL}' -OutFile $tmp -Label 'yt-dlp' -Start 2 -End 92; \
                 if ((Get-Item -LiteralPath $tmp).Length -le 0) {{ throw 'yt-dlp download is empty' }}; \
                 Write-VDLProgress 96 'yt-dlp 파일 배치 중'; \
                 Move-Item -LiteralPath $tmp -Destination '{target_quoted}' -Force; \
             }} finally {{ \
                 if (Test-Path -LiteralPath $tmp) {{ Remove-Item -LiteralPath $tmp -Force }}; \
             }}; \
             Write-Output 'yt-dlp installed'",
            download_helper = install_download_helper(),
        ),
        "aria2c" => install_zip_script(ARIA2_ZIP_URL, "aria2c.exe", &target),
        "ffmpeg" => install_zip_script(FFMPEG_ZIP_URL, "ffmpeg.exe", &target),
        _ => return Err(format!("알 수 없는 구성요소입니다: {name}")),
    };

    let _ = run_install_powershell(&app, &name, &script)?;
    emit_install_progress(&app, &name, 98.0, "실행 검증 중");
    validate_component_binary(&name, &target)?;
    emit_install_progress(&app, &name, 100.0, "설치 완료");
    Ok(format!("{} 설치 및 실행 검증 완료", component_label(&name)))
}

fn install_zip_script(url: &str, exe_name: &str, target: &Path) -> String {
    let target_quoted = ps_quote(target);
    let parent = target
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(bin_dir);
    let parent_quoted = ps_quote(&parent);
    let suffix = format!("{}_{}", std::process::id(), now_unix_millis());
    let temp_zip = format!("runtime_{}_{}.zip", exe_name.replace(".exe", ""), suffix);
    let temp_extract = format!("video_downloader_light_extract_{suffix}");
    format!(
        "{download_helper} \
         $ErrorActionPreference='Stop'; \
         [Net.ServicePointManager]::SecurityProtocol=[Net.SecurityProtocolType]::Tls12; \
         $zip = Join-Path $env:TEMP '{temp_zip}'; \
         $extract = Join-Path $env:TEMP '{temp_extract}'; \
         try {{ \
             if (Test-Path -LiteralPath $zip) {{ Remove-Item -LiteralPath $zip -Force }}; \
             if (Test-Path -LiteralPath $extract) {{ Remove-Item -LiteralPath $extract -Recurse -Force }}; \
             Save-VDLFile -Uri '{url}' -OutFile $zip -Label '{exe_name}' -Start 2 -End 78; \
             if ((Get-Item -LiteralPath $zip).Length -le 0) {{ throw '{exe_name} archive is empty' }}; \
             Write-VDLProgress 84 '{exe_name} 압축 해제 중'; \
             Expand-Archive -Path $zip -DestinationPath $extract -Force; \
             Write-VDLProgress 94 '{exe_name} 실행 파일 찾는 중'; \
             $exe = Get-ChildItem -Path $extract -Recurse -Filter '{exe_name}' | Select-Object -First 1; \
             if (-not $exe) {{ throw '{exe_name} not found in archive' }}; \
             New-Item -Path '{parent_quoted}' -ItemType Directory -Force | Out-Null; \
             Write-VDLProgress 96 '{exe_name} 파일 배치 중'; \
             Copy-Item -LiteralPath $exe.FullName -Destination '{target_quoted}' -Force; \
         }} finally {{ \
             if (Test-Path -LiteralPath $zip) {{ Remove-Item -LiteralPath $zip -Force }}; \
             if (Test-Path -LiteralPath $extract) {{ Remove-Item -LiteralPath $extract -Recurse -Force }}; \
         }}; \
         Write-Output '{exe_name} installed'",
        download_helper = install_download_helper(),
    )
}

fn install_download_helper() -> &'static str {
    r#"
function Write-VDLProgress([int]$Percent, [string]$Message) {
    Write-Output ("__VDL_INSTALL_PROGRESS__|{0}|{1}" -f $Percent, $Message)
}

function Save-VDLFile([string]$Uri, [string]$OutFile, [string]$Label, [int]$Start, [int]$End) {
    Write-VDLProgress $Start ("{0} 다운로드 준비 중" -f $Label)
    $request = [System.Net.HttpWebRequest]::Create($Uri)
    $request.AllowAutoRedirect = $true
    $request.UserAgent = "VideoDownloaderLight/0.1"
    $response = $request.GetResponse()
    try {
        $total = [int64]$response.ContentLength
        $inputStream = $response.GetResponseStream()
        $outputStream = [System.IO.File]::Create($OutFile)
        try {
            $buffer = New-Object byte[] 1048576
            $downloaded = [int64]0
            $lastPercent = -1
            while (($read = $inputStream.Read($buffer, 0, $buffer.Length)) -gt 0) {
                $outputStream.Write($buffer, 0, $read)
                $downloaded += $read
                if ($total -gt 0) {
                    $range = $End - $Start
                    $percent = $Start + [int][Math]::Floor(($downloaded * $range) / $total)
                    if ($percent -gt $End) { $percent = $End }
                    if ($percent -ne $lastPercent) {
                        Write-VDLProgress $percent ("{0} 다운로드 중 {1:n1}MB / {2:n1}MB" -f $Label, ($downloaded / 1MB), ($total / 1MB))
                        $lastPercent = $percent
                    }
                } elseif ($lastPercent -lt $Start) {
                    Write-VDLProgress $Start ("{0} 다운로드 중" -f $Label)
                    $lastPercent = $Start
                }
            }
        } finally {
            if ($outputStream) { $outputStream.Dispose() }
            if ($inputStream) { $inputStream.Dispose() }
        }
    } finally {
        if ($response) { $response.Dispose() }
    }
    Write-VDLProgress $End ("{0} 다운로드 완료" -f $Label)
}
"#
}

fn component_label(name: &str) -> &str {
    match name {
        "ytdlp" => "yt-dlp",
        "ffmpeg" => "ffmpeg",
        "aria2c" => "aria2c",
        _ => name,
    }
}

fn validate_component_binary(name: &str, target: &Path) -> Result<(), String> {
    if !target.is_file() {
        return Err(format!("{} 실행 파일이 없습니다.", component_label(name)));
    }

    if target
        .metadata()
        .map(|metadata| metadata.len())
        .unwrap_or(0)
        == 0
    {
        return Err(format!(
            "{} 실행 파일이 비어 있습니다.",
            component_label(name)
        ));
    }

    let mut command = Command::new(target);
    match name {
        "ffmpeg" => {
            command.arg("-version");
        }
        "ytdlp" | "aria2c" => {
            command.arg("--version");
        }
        _ => return Err(format!("알 수 없는 구성요소입니다: {name}")),
    }
    command.stdout(Stdio::null()).stderr(Stdio::null());
    let status = command
        .status()
        .map_err(|err| format!("{} 실행 검증 실패: {err}", component_label(name)))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!(
            "{} 실행 검증 실패: 종료 코드 {status}",
            component_label(name)
        ))
    }
}

fn normalize_host(host: &str) -> String {
    host.trim()
        .trim_end_matches('.')
        .to_ascii_lowercase()
        .strip_prefix("www.")
        .unwrap_or(&host.trim().trim_end_matches('.').to_ascii_lowercase())
        .to_string()
}

fn host_matches(host: &str, domain: &str) -> bool {
    let domain = normalize_host(domain);
    host == domain || host.ends_with(&format!(".{domain}"))
}

fn should_open_external_ad_url(url: &Url) -> bool {
    if !matches!(url.scheme(), "http" | "https") {
        return false;
    }
    let Some(host) = url.host_str().map(normalize_host) else {
        return false;
    };
    ["link.coupang.com", "coupang.com"]
        .iter()
        .any(|domain| host_matches(&host, domain))
}

fn open_external_url(url: &Url) {
    let _ = Command::new("rundll32.exe")
        .args(["url.dll,FileProtocolHandler", url.as_str()])
        .spawn();
}

#[tauri::command]
fn open_external_link(url: String) -> Result<(), String> {
    let parsed = Url::parse(&url).map_err(|_| "올바른 외부 링크가 아닙니다.".to_string())?;
    if !matches!(parsed.scheme(), "http" | "https") {
        return Err("http/https 링크만 열 수 있습니다.".to_string());
    }

    let host = parsed
        .host_str()
        .map(normalize_host)
        .ok_or_else(|| "도메인을 확인할 수 없습니다.".to_string())?;
    let allowed = ["buymeacoffee.com", "github.com"]
        .iter()
        .any(|domain| host_matches(&host, domain));
    if !allowed {
        return Err("허용되지 않은 외부 링크입니다.".to_string());
    }

    open_external_url(&parsed);
    Ok(())
}

fn validate_url_policy(input: &str) -> Result<String, String> {
    let parsed = Url::parse(input).map_err(|_| "올바른 URL이 아닙니다.".to_string())?;
    match parsed.scheme() {
        "http" | "https" => {}
        _ => return Err("http/https URL만 지원합니다.".to_string()),
    }

    let host = parsed
        .host_str()
        .map(normalize_host)
        .ok_or_else(|| "도메인을 확인할 수 없습니다.".to_string())?;

    if BLOCK_DOMAINS
        .iter()
        .any(|domain| host_matches(&host, domain))
    {
        return Err("유료 OTT/DRM 가능성이 높은 도메인은 정책상 차단됩니다.".to_string());
    }

    if !ALLOW_DOMAINS
        .iter()
        .any(|domain| host_matches(&host, domain))
    {
        return Err("지원 목록에 등록된 공개 플랫폼 URL만 허용됩니다.".to_string());
    }

    Ok(host)
}

fn format_selector(request: &DownloadRequest, has_ffmpeg: bool) -> String {
    if request.mode == "audio" {
        return "bestaudio/best".to_string();
    }

    if !has_ffmpeg {
        return "best[ext=mp4]/best".to_string();
    }

    let height_filter = if request.quality == "best" {
        String::new()
    } else {
        format!("[height<={}]", request.quality)
    };

    format!(
        "bestvideo*{height_filter}[ext=mp4]+bestaudio[ext=m4a]/bestvideo*{height_filter}+bestaudio/best{height_filter}/best"
    )
}

fn format_duration(seconds: Option<f64>) -> String {
    let Some(total_seconds) = seconds else {
        return "확인 중".to_string();
    };
    let total_seconds = total_seconds.max(0.0).round() as u64;
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;
    if hours > 0 {
        format!("{hours}:{minutes:02}:{seconds:02}")
    } else {
        format!("{minutes}:{seconds:02}")
    }
}

fn format_size(bytes: Option<u64>) -> String {
    let Some(bytes) = bytes else {
        return "계산 중".to_string();
    };
    const UNITS: [&str; 4] = ["B", "KB", "MB", "GB"];
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{} {}", bytes, UNITS[unit])
    } else {
        format!("{value:.1} {}", UNITS[unit])
    }
}

fn metadata_size_value(value: &serde_json::Value) -> Option<(u64, bool)> {
    value
        .get("filesize")
        .and_then(serde_json::Value::as_u64)
        .map(|size| (size, false))
        .or_else(|| {
            value
                .get("filesize_approx")
                .and_then(serde_json::Value::as_u64)
                .map(|size| (size, true))
        })
}

fn pick_metadata_size(metadata: &serde_json::Value) -> (Option<u64>, bool) {
    if let Some(requested_formats) = metadata
        .get("requested_formats")
        .and_then(serde_json::Value::as_array)
    {
        let mut total = 0_u64;
        let mut found = false;
        let mut approximate = false;
        for format in requested_formats {
            if let Some((size, is_approximate)) = metadata_size_value(format) {
                total = total.saturating_add(size);
                found = true;
                approximate |= is_approximate;
            }
        }
        if found {
            return (Some(total), approximate);
        }
    }

    if let Some((size, approximate)) = metadata_size_value(metadata) {
        return (Some(size), approximate);
    }

    if let Some(max_size) = metadata
        .get("formats")
        .and_then(serde_json::Value::as_array)
        .and_then(|formats| {
            formats
                .iter()
                .filter_map(|format| metadata_size_value(format).map(|(size, _)| size))
                .max()
        })
    {
        return (Some(max_size), true);
    }

    (None, true)
}

fn run_metadata_command(mut command: Command, timeout: Duration) -> Option<Vec<u8>> {
    let stdout_path = env::temp_dir().join(format!(
        "vdl-metadata-{}-{}.json",
        std::process::id(),
        now_unix_seconds()
    ));
    let stdout_file = File::create(&stdout_path).ok()?;
    command.stdout(Stdio::from(stdout_file));
    command.stderr(Stdio::null());

    let mut child = command.spawn().ok()?;
    let started_at = Instant::now();
    let status = loop {
        if let Ok(Some(status)) = child.try_wait() {
            break status;
        }

        if started_at.elapsed() > timeout {
            let _ = child.kill();
            let _ = child.wait();
            let _ = fs::remove_file(&stdout_path);
            return None;
        }

        thread::sleep(Duration::from_millis(120));
    };

    if !status.success() {
        let _ = fs::remove_file(&stdout_path);
        return None;
    }

    let output = fs::read(&stdout_path).ok();
    let _ = fs::remove_file(&stdout_path);
    output
}

fn fetch_download_metadata(
    ytdlp: &Path,
    request: &DownloadRequest,
    has_ffmpeg: bool,
    history_id: &str,
) -> DownloadMetadataPayload {
    let mut command = Command::new(ytdlp);
    command
        .arg("--ignore-config")
        .arg("--no-playlist")
        .arg("--dump-single-json")
        .arg("--skip-download")
        .arg("--no-warnings")
        .arg("-f")
        .arg(format_selector(request, has_ffmpeg))
        .arg(&request.url);
    prepend_bin_to_path(&mut command);

    let Some(output) = run_metadata_command(command, Duration::from_secs(12)) else {
        return default_download_metadata(history_id);
    };

    let Ok(metadata) = serde_json::from_slice::<serde_json::Value>(&output) else {
        return default_download_metadata(history_id);
    };

    let title = metadata
        .get("title")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("제목 확인 중")
        .to_string();
    let duration = format_duration(metadata.get("duration").and_then(serde_json::Value::as_f64));
    let (size_bytes, size_approximate) = pick_metadata_size(&metadata);
    let size = format_size(size_bytes);
    let thumbnail = metadata
        .get("thumbnail")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("")
        .to_string();

    DownloadMetadataPayload {
        history_id: history_id.to_string(),
        title,
        duration,
        size,
        thumbnail,
        size_approximate,
    }
}

fn default_download_metadata(history_id: &str) -> DownloadMetadataPayload {
    DownloadMetadataPayload {
        history_id: history_id.to_string(),
        title: "영상 정보를 확인 중입니다.".to_string(),
        duration: "확인 중".to_string(),
        size: "계산 중".to_string(),
        thumbnail: String::new(),
        size_approximate: true,
    }
}
fn parse_percent(line: &str) -> Option<f64> {
    let percent_index = line.find('%')?;
    let before = &line[..percent_index];
    let token = before.split_whitespace().last()?.trim();
    token.parse::<f64>().ok()
}

fn prepend_bin_to_path(command: &mut Command) {
    let mut paths = vec![bin_dir()];
    if let Some(existing) = env::var_os("PATH") {
        paths.extend(env::split_paths(&existing));
    }
    if let Ok(joined) = env::join_paths(paths) {
        command.env("PATH", joined);
    }
}

fn reported_output_path(line: &str) -> Option<PathBuf> {
    let trimmed = line.trim().trim_matches('"');
    if trimmed.is_empty() || trimmed.starts_with('[') || trimmed.contains("://") {
        return None;
    }

    let path = PathBuf::from(trimmed);
    if path.is_absolute() {
        Some(path)
    } else {
        None
    }
}

fn is_valid_download_file(path: &Path) -> bool {
    if !path.is_file() {
        return false;
    }

    let lower_name = path
        .file_name()
        .map(|name| name.to_string_lossy().to_ascii_lowercase())
        .unwrap_or_default();
    if lower_name.ends_with(".part")
        || lower_name.ends_with(".ytdl")
        || lower_name.ends_with(".tmp")
        || lower_name.ends_with(".temp")
    {
        return false;
    }

    path.metadata()
        .map(|metadata| metadata.len() > 0)
        .unwrap_or(false)
}

fn is_recent_enough(modified: SystemTime, started_at: SystemTime) -> bool {
    modified.duration_since(started_at).is_ok()
        || started_at
            .duration_since(modified)
            .map(|duration| duration.as_secs() <= 5)
            .unwrap_or(false)
}

fn find_recent_download_file(save_dir: &Path, started_at: SystemTime) -> Option<PathBuf> {
    let entries = fs::read_dir(save_dir).ok()?;
    let mut best: Option<(SystemTime, PathBuf)> = None;

    for entry in entries.flatten() {
        let path = entry.path();
        if !is_valid_download_file(&path) {
            continue;
        }
        let modified = entry
            .metadata()
            .and_then(|metadata| metadata.modified())
            .unwrap_or(UNIX_EPOCH);
        if !is_recent_enough(modified, started_at) {
            continue;
        }
        if best
            .as_ref()
            .map(|(best_modified, _)| modified > *best_modified)
            .unwrap_or(true)
        {
            best = Some((modified, path));
        }
    }

    best.map(|(_, path)| path)
}

fn verify_download_output(
    reported_files: &[PathBuf],
    save_dir: &Path,
    started_at: SystemTime,
) -> Option<PathBuf> {
    for path in reported_files {
        if is_valid_download_file(path) {
            return Some(path.clone());
        }
    }

    find_recent_download_file(save_dir, started_at)
}

#[tauri::command]
fn start_download(app: AppHandle, request: DownloadRequest) -> Result<(), String> {
    if !read_settings().legal_accepted {
        return Err("법적 고지에 동의한 후 다운로드할 수 있습니다.".to_string());
    }

    let host = validate_url_policy(&request.url)?;
    let ytdlp = component_path("ytdlp")?;
    if validate_component_binary("ytdlp", &ytdlp).is_err() {
        return Err("yt-dlp가 필요합니다. 앱을 다시 실행해 자동 설치를 완료하세요.".to_string());
    }

    let ffmpeg = component_path("ffmpeg")?;
    let aria2c = component_path("aria2c")?;
    let has_ffmpeg = validate_component_binary("ffmpeg", &ffmpeg).is_ok();
    let has_aria2c = validate_component_binary("aria2c", &aria2c).is_ok();
    let save_dir = if request.save_dir.trim().is_empty() {
        default_save_dir()
    } else {
        PathBuf::from(request.save_dir.trim())
    };
    fs::create_dir_all(&save_dir).map_err(|err| format!("저장 폴더 생성 실패: {err}"))?;
    let history_id = append_history(&request.url, &save_dir)?;
    let _ = app.emit("history-updated", read_history());

    let app_for_thread = app.clone();
    thread::spawn(move || {
        let _ = app_for_thread.emit("download-log", format!("[정책] 통과: {host}"));
        let metadata = fetch_download_metadata(&ytdlp, &request, has_ffmpeg, &history_id);
        let _ = app_for_thread.emit("download-metadata", metadata);
        let download_started_at = SystemTime::now();

        let out_template = save_dir.join("%(title).200B [%(id)s].%(ext)s");
        let format_selector = format_selector(&request, has_ffmpeg);
        let mut command = Command::new(&ytdlp);
        command
            .arg("--ignore-config")
            .arg("--no-playlist")
            .arg("--newline")
            .arg("--progress")
            .arg("--print")
            .arg("after_move:filepath")
            .arg("--no-warnings")
            .arg("--windows-filenames")
            .arg("--continue")
            .arg("--retries")
            .arg("10")
            .arg("--extractor-retries")
            .arg("3")
            .arg("--fragment-retries")
            .arg("10")
            .arg("--retry-sleep")
            .arg("http:linear=1::2")
            .arg("--retry-sleep")
            .arg("fragment:exp=1:20")
            .arg("--concurrent-fragments")
            .arg("10")
            .arg("-f")
            .arg(format_selector)
            .arg("-o")
            .arg(out_template.to_string_lossy().as_ref());

        if has_aria2c {
            command
                .arg("--downloader")
                .arg("http,https,ftp:aria2c")
                .arg("--downloader-args")
                .arg("aria2c:-x8 -s8 -k1M --summary-interval=0 --console-log-level=warn");
            let _ = app_for_thread.emit(
                "download-log",
                "[정보] 일반 HTTP 계열은 aria2c 고속 모드를 사용합니다.",
            );
        }

        if has_ffmpeg {
            command
                .arg("--ffmpeg-location")
                .arg(bin_dir().to_string_lossy().as_ref());
            if request.mode != "audio" {
                command.arg("--merge-output-format").arg("mp4");
            } else if request.mode == "audio" {
                command.arg("-x").arg("--audio-format").arg("mp3");
                if request.audio_quality == "best" {
                    command.arg("--audio-quality").arg("0");
                } else {
                    command
                        .arg("--audio-quality")
                        .arg(format!("{}K", request.audio_quality));
                }
            }
        } else {
            let _ = app_for_thread.emit(
                "download-log",
                "[알림] ffmpeg 미설치: 병합/변환이 필요한 최고 품질 형식은 제한됩니다.",
            );
        }

        command.arg(&request.url);
        command.stdout(Stdio::piped()).stderr(Stdio::piped());
        prepend_bin_to_path(&mut command);

        let mut child = match command.spawn() {
            Ok(child) => child,
            Err(err) => {
                update_history_status(&history_id, "실패");
                let _ = app_for_thread.emit("history-updated", read_history());
                let _ = app_for_thread.emit("download-failed", format!("yt-dlp 실행 실패: {err}"));
                return;
            }
        };

        let stdout = match child.stdout.take() {
            Some(stdout) => stdout,
            None => {
                update_history_status(&history_id, "실패");
                let _ = app_for_thread.emit("history-updated", read_history());
                let _ = app_for_thread
                    .emit("download-failed", "yt-dlp 출력 스트림을 읽을 수 없습니다.");
                return;
            }
        };

        let stderr_lines: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
        let stderr_handle = child.stderr.take().map(|stderr| {
            let app_for_stderr = app_for_thread.clone();
            let stderr_lines = Arc::clone(&stderr_lines);
            thread::spawn(move || {
                for line in BufReader::new(stderr).lines().map_while(Result::ok) {
                    let trimmed = line.trim().to_string();
                    if trimmed.is_empty() {
                        continue;
                    }
                    let _ = app_for_stderr.emit("download-log", trimmed.clone());
                    if let Ok(mut lines) = stderr_lines.lock() {
                        if lines.len() >= 80 {
                            lines.remove(0);
                        }
                        lines.push(trimmed);
                    }
                }
            })
        });

        let mut last_line = String::new();
        let mut reported_files: Vec<PathBuf> = Vec::new();
        for line in BufReader::new(stdout).lines().map_while(Result::ok) {
            let trimmed = line.trim().to_string();
            if trimmed.is_empty() {
                continue;
            }
            last_line = trimmed.clone();
            if let Some(path) = reported_output_path(&trimmed) {
                reported_files.push(path);
            }
            let _ = app_for_thread.emit("download-log", trimmed.clone());

            let lowered = trimmed.to_ascii_lowercase();
            if lowered.contains("drm")
                && (lowered.contains("protected")
                    || lowered.contains("unsupported")
                    || lowered.contains("not supported")
                    || lowered.contains("license"))
            {
                let _ = child.kill();
                let _ = child.wait();
                if let Some(handle) = stderr_handle {
                    let _ = handle.join();
                }
                update_history_status(&history_id, "차단됨");
                let _ = app_for_thread.emit("history-updated", read_history());
                let _ = app_for_thread.emit(
                    "download-failed",
                    "DRM 보호 신호가 감지되어 다운로드를 차단했습니다.",
                );
                return;
            }

            if let Some(percent) = parse_percent(&trimmed) {
                let _ = app_for_thread.emit(
                    "download-progress",
                    ProgressPayload {
                        percent,
                        message: "다운로드 중...".to_string(),
                    },
                );
            }
        }

        let status = child.wait();
        if let Some(handle) = stderr_handle {
            let _ = handle.join();
        }
        let stderr_text = stderr_lines
            .lock()
            .map(|lines| lines.join("\n"))
            .unwrap_or_default();

        match status {
            Ok(status) if status.success() => {
                if let Some(output_file) =
                    verify_download_output(&reported_files, &save_dir, download_started_at)
                {
                    update_history_status(&history_id, "성공");
                    let _ = app_for_thread.emit("history-updated", read_history());
                    let _ = app_for_thread.emit(
                        "download-finished",
                        format!("다운로드 완료: {}", output_file.to_string_lossy()),
                    );
                } else {
                    update_history_status(&history_id, "실패");
                    let _ = app_for_thread.emit("history-updated", read_history());
                    let _ = app_for_thread.emit(
                        "download-failed",
                        "다운로드 프로세스는 종료됐지만 출력 파일을 확인하지 못했습니다.",
                    );
                }
            }
            Ok(status) => {
                let detail = if !stderr_text.trim().is_empty() {
                    stderr_text.trim().to_string()
                } else if !last_line.is_empty() {
                    last_line
                } else {
                    format!("yt-dlp 종료 코드: {status}")
                };
                update_history_status(&history_id, "실패");
                let _ = app_for_thread.emit("history-updated", read_history());
                let _ = app_for_thread.emit("download-failed", detail);
            }
            Err(err) => {
                update_history_status(&history_id, "실패");
                let _ = app_for_thread.emit("history-updated", read_history());
                let _ =
                    app_for_thread.emit("download-failed", format!("yt-dlp 종료 확인 실패: {err}"));
            }
        }
    });

    Ok(())
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            if let Some(window) = app.get_webview_window("main") {
                let icon = tauri::image::Image::new(
                    include_bytes!("../icons/icon-256x256.rgba"),
                    256,
                    256,
                );
                window.set_icon(icon)?;
            }
            Ok(())
        })
        .plugin(
            tauri::plugin::Builder::<tauri::Wry, ()>::new("external-ad-navigation")
                .on_navigation(|_, url| {
                    if should_open_external_ad_url(url) {
                        open_external_url(url);
                        false
                    } else {
                        true
                    }
                })
                .build(),
        )
        .invoke_handler(tauri::generate_handler![
            get_state,
            save_settings,
            accept_legal,
            delete_history_record,
            clear_history,
            install_component,
            choose_download_dir,
            open_external_link,
            start_download
        ])
        .run(tauri::generate_context!())
        .expect("failed to run app");
}
